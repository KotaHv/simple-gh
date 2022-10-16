#!/bin/sh
exec /simple-gh
if [ "${DEBUG}" = "true" ]; then
    while true
    do
        echo "exec /simple-gh fail"
        sleep 1000
    done
fi