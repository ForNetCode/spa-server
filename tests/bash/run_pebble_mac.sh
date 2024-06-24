#!/bin/bash
#change it to self
export IP=192.168.1.255
docker run -p 14000:14000 -p 15000:15000 --rm \
--add-host=local.fornetcode.com:$(IP) \
--name pebble \
ghcr.io/letsencrypt/pebble:2.6.0
