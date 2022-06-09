#!/bin/bash
echo "Script executed from: ${PWD}"

BASEDIR=$(dirname $0)
echo "Script location: ${BASEDIR}"
cd $(dirname $0)
# it would be shutdown, so you need to start munually
# dfx start --background
