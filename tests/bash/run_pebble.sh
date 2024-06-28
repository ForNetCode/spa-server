#!/bin/bash
docker run --network=host -d -e PEBBLE_WFE_NONCEREJECT=0 \
--name pebble \
-v $(pwd)/../data/pebble/pebble_config.json:/test/config/pebble_config.json \
ghcr.io/letsencrypt/pebble:2.6.0 -config /test/config/pebble_config.json
