
set -e
# pass root vault token in to access all secrets
[ -n "$VAULT_TOKEN" ] || { echo VAULT_TOKEN environment variable is required; exit 7; }
export PGHOST=master.epl-pg-testdb.service.consul
export PGPORT=5433
export PGUSER=postgres
export PGPASSWORD=$( vault kv get -field=pg_superuser_password epl/pg/testdb )
export PGDATABASE=postgres
while ! psql -c 'SELECT 1'
do
    echo Waiting for database deployment testdb to be up...
    sleep 5
done
export PG_EXPORTER_PASSWORD=$( vault kv get -field=pg_exporter_password epl/pg/testdb )
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
export THIS_DB_PW=$( vault kv get -field=pg_db_grafana_password epl/pg/testdb )
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

export THIS_DB_PW=$( vault kv get -field=pg_db_bbtest_password epl/pg/testdb )
echo "SELECT 'CREATE DATABASE bbtest' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'bbtest')\gexec" | psql
cat <<EOF | psql -f -

DO
\$\$
BEGIN
  IF NOT EXISTS (SELECT * FROM pg_user WHERE usename = 'bbtest') THEN
     CREATE USER bbtest password '$THIS_DB_PW';
  END IF;
  GRANT ALL PRIVILEGES ON DATABASE bbtest TO bbtest;
  ALTER DATABASE bbtest OWNER TO bbtest;
END
\$\$
;

EOF

export THIS_DB_PW=$( vault kv get -field=pg_db_testdb_a_password epl/pg/testdb )
echo "SELECT 'CREATE DATABASE testdb_a' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'testdb_a')\gexec" | psql
cat <<EOF | psql -f -

DO
\$\$
BEGIN
  IF NOT EXISTS (SELECT * FROM pg_user WHERE usename = 'testdb_a') THEN
     CREATE USER testdb_a password '$THIS_DB_PW';
  END IF;
  GRANT ALL PRIVILEGES ON DATABASE testdb_a TO testdb_a;
  ALTER DATABASE testdb_a OWNER TO testdb_a;
END
\$\$
;

EOF

PGUSER=testdb_a PGPASSWORD=$THIS_DB_PW PGDATABASE=testdb_a pg-migrations/up_testdb.sh &

echo Migrations scheduled, waiting for finish...
wait
echo All migrations ran successfully
