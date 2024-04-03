#!/bin/bash

source ./.env

# run on windows host
espflash.exe --monitor ${SERIAL_PORT} $(wslpath -w "${1}")