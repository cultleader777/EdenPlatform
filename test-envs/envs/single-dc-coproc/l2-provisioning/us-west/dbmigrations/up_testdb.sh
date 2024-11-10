#!/bin/sh
set -e

[ -n "$PGHOST" ] || { echo "PGHOST environment variable must be provided"; exit 7; }
[ -n "$PGPORT" ] || { echo "PGPORT environment variable must be provided"; exit 7; }
[ -n "$PGDATABASE" ] || { echo "PGDATABASE environment variable must be provided"; exit 7; }
[ -n "$PGUSER" ] || { echo "PGUSER environment variable must be provided"; exit 7; }
[ -n "$PGPASSWORD" ] || { echo "PGPASSWORD environment variable must be provided"; exit 7; }

while ! psql -c 'SELECT 1'
do
    echo Waiting for database to be up...
    sleep 5
done

psql -c 'CREATE TABLE IF NOT EXISTS epl_schema_migrations(logical_time INT PRIMARY KEY, time_started TIMESTAMP, time_ended TIMESTAMP);'
if ! psql -c "SELECT 'MIG_FOUND' FROM epl_schema_migrations WHERE logical_time = 1" | grep MIG_FOUND
then
    cat <<EOF | psql -f -
    BEGIN;
    INSERT INTO epl_schema_migrations(logical_time, time_started) VALUES (1, NOW());

          CREATE TABLE foo (
            id INT PRIMARY KEY
          );
        
    UPDATE epl_schema_migrations SET time_ended = clock_timestamp() WHERE logical_time = 1;
    COMMIT;
EOF
fi
