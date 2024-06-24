#!/bin/bash
docker run --network=host -d
--name pebble \
ghcr.io/letsencrypt/pebble:2.6.0  && \
exit 0;

## next command is for mac test.
export IP=192.168.1.255 #change it to self
docker run -p 14000:14000 -p 15000:15000 --rm \
--add-host=local.fornetcode.com:$(IP) \
--name pebble \
ghcr.io/letsencrypt/pebble:2.6.0
