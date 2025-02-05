# FROM postgres:17.2-bookworm
FROM postgres@sha256:87ec5e0a167dc7d4831729f9e1d2ee7b8597dcc49ccd9e43cc5f89e808d2adae

# get latest libpq version like so
# apt-get update
# apt-cache madison libpq-dev
# apt-cache madison python3-pip

RUN apt-get update && \
    apt-get install -y python3-pip=23.0.1+dfsg-1 libpq-dev=17.2-1.pgdg120+1 && \
    rm -rf /var/lib/apt/lists/*

RUN pip install --break-system-packages 'psycopg2==2.9.10' && \
    pip install --break-system-packages 'patroni[consul]==4.0.4'

USER postgres
