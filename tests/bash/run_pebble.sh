#!/bin/bash
docker run --network=host -d -e PEBBLE_WFE_NONCEREJECT=0 \
--name pebble \
ghcr.io/letsencrypt/pebble:2.6.0
