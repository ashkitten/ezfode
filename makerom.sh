#! /usr/bin/env bash
set -ex
cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/ezfode target/ezfode.gba
~/.cargo/bin/gbafix -p -tezfode -cEZFO -mRS target/ezfode.gba
