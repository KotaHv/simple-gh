# simple-gh

## docker-compose file example
```
version: "3"

services:
  simple-gh:
    image: kotahv/simple-gh:latest
    container_name: simple-gh
    hostname: simple-gh
    environment:
      - simple_gh_token=123abcd
      - simple_gh_title=simple-gh
      - simple_gh_max_cache=512MiB # 536870912 | 512MB Mib: 1024*1024 MB: 1000*1000
      - simple_gh_file_max=20MiB
      - simple_gh_cache_time=3600 # seconds
      - simple_gh_data_dir=data
      - simple_gh_log_dir=data/logs
      - simple_gh_cache_dir=data/cache
    volumes:
      - <Your Path>:/opt/simple-gh/data
    restart: unless-stopped
    ports:
      - 12345:80
```