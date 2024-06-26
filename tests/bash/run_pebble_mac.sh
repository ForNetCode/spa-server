#!/bin/bash
#change it to self
#export IP=192.168.1.255
docker run -p 14000:14000 -p 15000:15000 --rm -e PEBBLE_WFE_NONCEREJECT=0 \
--add-host=local.fornetcode.com:$IP \
--add-host=local2.fornetcode.com:$IP \
--name pebble \
-v $(pwd)/../data/pebble/pebble_config.json:/test/config/pebble_config.json \
ghcr.io/letsencrypt/pebble:2.6.0 -config /test/config/pebble_config.json
