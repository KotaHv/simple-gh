# simple-gh

## docker-compose file example
```
version: "3"

services:
  simple-gh:
    image: kotahv/simple-gh:rocket
    container_name: simple-gh
    hostname: simple-gh
    environment:
      - SIMPLE_GH_TOKEN=123abcd
      - SIMPLE_GH_LOG_LEVEL=info
      - SIMPLE_GH_LOG_STYLE=auto
      - SIMPLE_GH_FILE_MAX=20MiB
      - SIMPLE_GH_CACHE_MAX=512MiB # 536870912 | 512MB Mib: 1024*1024 MB: 1000*1000
      - SIMPLE_GH_CACHE_EXPIRY=3600 # seconds
    volumes:
      - <Your Path>:/cache
    restart: unless-stopped
    ports:
      - 12345:80
```