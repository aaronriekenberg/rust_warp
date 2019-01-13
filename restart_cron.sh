#!/bin/sh

pgrep rust_warp > /dev/null 2>&1
if [ $? -eq 1 ]; then
  cd ~/rust_warp
  ./restart.sh > /dev/null 2>&1
fi
