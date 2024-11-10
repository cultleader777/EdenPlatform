
set -e
# pass root vault token in to access all secrets
[ -n "$VAULT_TOKEN" ] || { echo VAULT_TOKEN environment variable is required; exit 7; }
export PGHOST=master.epl-pg-testdb-us-west.service.consul
export PGPORT=5433
export PGUSER=postgres
export PGPASSWORD=$( vault kv get -field=pg_superuser_password epl/pg/testdb-us-west )
export PGDATABASE=postgres
while ! psql -c 'SELECT 1'
do
    echo Waiting for database deployment testdb-us-west to be up...
    sleep 5
done
export PG_EXPORTER_PASSWORD=$( vault kv get -field=pg_exporter_password epl/pg/testdb-us-west )
cat <<EOF | psql -f -


CREATE OR REPLACE FUNCTION __tmp_create_user() returns void as \$\$
BEGIN
  IF NOT EXISTS (
          SELECT                       -- SELECT list can stay empty for this
          FROM   pg_catalog.pg_user
          WHERE  usename = 'postgres_exporter') THEN
    CREATE USER postgres_exporter;
  END IF;
END;
\$\$ language plpgsql;

SELECT __tmp_create_user();
DROP FUNCTION __tmp_create_user();

ALTER USER postgres_exporter WITH PASSWORD '$PG_EXPORTER_PASSWORD';
ALTER USER postgres_exporter SET SEARCH_PATH TO postgres_exporter,pg_catalog;

GRANT CONNECT ON DATABASE postgres TO postgres_exporter;
GRANT pg_monitor to postgres_exporter;

EOF
export THIS_DB_PW=$( vault kv get -field=pg_db_grafana_password epl/pg/testdb-us-west )
echo "SELECT 'CREATE DATABASE grafana' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'grafana')\gexec" | psql
cat <<EOF | psql -f -

DO
\$\$
BEGIN
  IF NOT EXISTS (SELECT * FROM pg_user WHERE usename = 'grafana') THEN
     CREATE USER grafana password '$THIS_DB_PW';
  END IF;
  GRANT ALL PRIVILEGES ON DATABASE grafana TO grafana;
  ALTER DATABASE grafana OWNER TO grafana;
END
\$\$
;

EOF

echo Migrations scheduled, waiting for finish...
wait
echo All migrations ran successfully
