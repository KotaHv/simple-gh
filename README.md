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
      - simple_gh_token=123
      - simple_gh_openapi_url=
      - simple_gh_title=simple-gh
    volumes:
      - /my/own/logs:/opt/simple-gh/logs
    restart: unless-stopped
    ports:
      - 12345:80
```