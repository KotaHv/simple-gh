#!/bin/sh
curl --insecure --fail --silent --show-error \
     "http://localhost/alive" || exit 1
