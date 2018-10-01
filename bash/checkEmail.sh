#! /bin/bash

# Set useful variables
EMAIL="$1";
DOMAIN=`cut -d "@" -f 2 <<< "$1"`;
MX_HOSTS=(`nslookup -q=mx $DOMAIN | grep mail | cut -d " " -f 5`) # Find MX hosts of our domain
PORTS=(25 587 465); # The PORTS to test our telnet on, in this order

if [[ $# == 2 ]]; then
    SENDER="$2"
fi
if [ -z $SENDER ]; then
    SENDER='email@example.com'
fi

for HOST in ${MX_HOSTS[@]}; do
    for PORT in ${PORTS[@]}; do
        OUTPUT=`expect expectTelnet.tcl $EMAIL $HOST $PORT $SENDER 2>/dev/null`;
        if [[ `echo $OUTPUT | grep "2.1.5"` ]]; then # 2.1.5 means address exists
            echo true;
            exit 0;
        fi
    done
done

echo false;
exit 1;
