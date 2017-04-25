#! /bin/bash

# Set useful variables
EMAIL="$1";
HOST=`cut -d "@" -f 2 <<< "$1"`;
PORTS=(587 25 465); # The PORTS to test our telnet on, in this order

for PORT in ${PORTS[@]}
do
  OUTPUT=`expect expectTelnet.sh $EMAIL $HOST $PORT`;
  if [[ `echo $OUTPUT | grep "2.1.5"` ]]; # 2.1.5 means address exists
  then
    echo true;
    exit 0;
  elif [[ `echo $OUTPUT | grep "5."` ]]; # 5.*.* are error codes
  then
    echo false;
    exit 1
  fi
done

echo false;
exit 1;
