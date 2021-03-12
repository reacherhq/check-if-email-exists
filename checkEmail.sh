#! /bin/bash

# Set useful variables
EMAIL="$1";
DOMAIN=$(cut -d "@" -f 2 <<< "$1";)
MX_HOSTS=$(nslookup -q=mx "$DOMAIN" | grep mail | cut -d " " -f 5 | head -n 1 | sed -e 's/\.$//';) # Find MX hosts of our domain ,remove dot and only one of MX-Record.
PORTS=(25 587 465); # The PORTS to test our telnet on, in this order

if [[ $# == 2 ]]; then
    SENDER="$2"
    DOMAIN_SENDER=$(cut -d "@" -f 2 <<< "$SENDER";)
    MX_HOSTS_SENDER=$(nslookup -q=mx "$DOMAIN_SENDER" | grep mail | cut -d " " -f 5 | head -n 1 | sed -e 's/\.$//';) # Find MX hosts of our sender domain, remove dot and only one of MX-Record.
fi
if [ -z "$SENDER" ]; then
    SENDER='email@example.com'
fi

for HOST in "${MX_HOSTS[@]}"; do
    for PORT in "${PORTS[@]}"; do
        OUTPUT=$(expect expectTelnet.tcl "$EMAIL" "$HOST" "$PORT" "$SENDER" "$MX_HOSTS_SENDER" 2>/dev/null;) # add EHLO (MX_HOSTS_SENDER) from sender.
        if [[ $(echo "$OUTPUT" | grep -i "250 OK") ]]; then # 2.1.5 -> deprecated on most mail server.
            echo true;
            exit 0;
        fi
    done
done

echo false;
exit 1;
