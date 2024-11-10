# FROM postgres:15.1-bullseye
FROM postgres@sha256:766e8867182b474f02e48c7b1a556d12ddfa246138ddc748d70c891bf2873d82

RUN apt-get update && \
    apt-get install -y python3-pip=20.3.4-4+deb11u1 libpq-dev=15.1-1.pgdg110+1 && \
    rm -rf /var/lib/apt/lists/*

RUN pip install 'psycopg2==2.9.5' && \
    pip install 'patroni[consul]==2.1.4'

USER postgres
