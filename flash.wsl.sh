#!/bin/bash

source ./.env

# run on windows host
espflash.exe ${SERIAL_PORT} $(wslpath -w "${1}")