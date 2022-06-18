FROM alpine:3.16 AS base

RUN apk add --no-cache --update python3 tzdata curl

FROM python:3.10 AS install

COPY requirements.txt .

RUN pip install -r requirements.txt --no-binary pydantic

RUN rm -rf /usr/local/lib/python3.10/site-packages/pip*

RUN rm -rf /usr/local/lib/python3.10/site-packages/setuptools*

FROM base

COPY --from=install /usr/local/lib/python3.10/site-packages /usr/lib/python3.10/site-packages

WORKDIR /opt/simple-gh

COPY . .

ENTRYPOINT ["python3", "-m", "uvicorn", "main:app", "--proxy-headers", "--host", "0.0.0.0", "--port", "80", "--no-access-log"]

EXPOSE 80

HEALTHCHECK --interval=5s --timeout=3s --start-period=30s \
    CMD curl --insecure --fail --silent --show-error http://localhost/healthcheck || exit 1