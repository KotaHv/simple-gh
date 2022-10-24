FROM --platform=$BUILDPLATFORM python:3.10-slim as requirements-stage

WORKDIR /tmp

RUN pip install poetry

COPY ./pyproject.toml ./poetry.lock* /tmp/

RUN poetry export -f requirements.txt --output requirements.txt --without-hashes


FROM python:3.10-slim

WORKDIR /app

#COPY ./requirements.txt .
COPY --from=requirements-stage /tmp/requirements.txt .

RUN pip install --no-cache-dir --upgrade -r /app/requirements.txt

COPY . .

COPY docker/gunicorn_conf.py /gunicorn_conf.py
COPY docker/healthcheck.sh /healthcheck.sh
COPY docker/start.sh /start.sh
COPY docker/start-reload.sh /start-reload.sh

RUN mkdir /app/data
VOLUME /app/data

ENV PYTHONPATH=/app

EXPOSE 80

HEALTHCHECK --interval=10s --timeout=5s CMD ["/healthcheck.sh"]

CMD [ "/start.sh" ]