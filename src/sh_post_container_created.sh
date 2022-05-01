#!/bin/bash
echo "Script executed from: ${PWD}"

BASEDIR=$(dirname $0)
echo "Script location: ${BASEDIR}"
cd $(dirname $0)
#git config --unset --global http.proxy
#git config --unset --global https.proxy
#git config --unset --global core.gitproxy
dfx start --background
./sh_setup_dev.sh
dfx stop
