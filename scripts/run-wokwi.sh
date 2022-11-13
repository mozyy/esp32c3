#!/usr/bin/env bash

set -e

BUILD_MODE=""
case "$1" in
"" | "release")
    bash scripts/build.sh
    BUILD_MODE="release"
    ;;
"debug")
    bash scripts/build.sh debug
    BUILD_MODE="debug"
    ;;
*)
    echo "Wrong argument. Only \"debug\"/\"release\" arguments are supported"
    exit 1
    ;;
esac

if [ "${USER}" == "gitpod" ]; then
    gp_url=$(gp url 9012)
    echo "gp_url=${gp_url}"
    export WOKWI_HOST=${gp_url:8}
elif [ "${CODESPACE_NAME}" != "" ]; then
    export WOKWI_HOST=${CODESPACE_NAME}-9012.githubpreview.dev
fi

export ESP_ARCH=riscv32imac-unknown-none-elf

# TODO: Update with your Wokwi Project
export WOKWI_PROJECT_ID="347318324077527635"
if [ "${WOKWI_PROJECT_ID}" == "" ]; then
    wokwi-server --chip esp32c3 target/${ESP_ARCH}/${BUILD_MODE}/wifi
else
    wokwi-server --chip esp32c3 --id ${WOKWI_PROJECT_ID} target/${ESP_ARCH}/${BUILD_MODE}/wifi
fi
