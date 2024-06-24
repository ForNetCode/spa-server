#!/bin/bash
docker run --network=host -d \
--name pebble \
ghcr.io/letsencrypt/pebble:2.6.0
