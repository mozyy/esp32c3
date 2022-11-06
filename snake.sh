#!/usr/bin/env bash


if [ "${CODESPACE_NAME}" != "" ]; then
    export WOKWI_HOST=${CODESPACE_NAME}-9012.githubpreview.dev
fi

export ESP_ARCH=riscv32imac-unknown-none-elf
export BUILD_MODE="release"

cargo build --bin game --${BUILD_MODE}


wokwi-server --chip esp32c3 --id 345825844119208530 target/${ESP_ARCH}/${BUILD_MODE}/game
