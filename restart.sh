#!/bin/sh -x

CONFIG_FILE=config/$(hostname)-config.json

pkill rust_warp

nohup ./target/release/rust_warp $CONFIG_FILE 2>&1 | svlogd logs &
