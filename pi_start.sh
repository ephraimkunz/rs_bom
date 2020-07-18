#!/bin/bash
# To be run on the Raspberry Pi to start the server.

if [[ $UID != 0 ]]; then
    echo "Please run this script with sudo:"
    echo "sudo $0 $*"
    exit 1
fi

ROCKET_PORT=80 nohup ./rs_bom_api >log.txt 2>&1 &