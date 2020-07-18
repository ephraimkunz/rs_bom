#!/bin/sh
cross build --release --target armv7-unknown-linux-gnueabihf
scp target/armv7-unknown-linux-gnueabihf/release/rs_bom_api pi@73.170.106.70:~