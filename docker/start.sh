#!/bin/sh
python3 -m uvicorn main:app --proxy-headers --host 0.0.0.0 --port 80 --no-access-log
if [ "${DEBUG}" = "true" ]; then
    while true
    do
        echo "exec /simple-gh fail"
        sleep 1000
    done
fi