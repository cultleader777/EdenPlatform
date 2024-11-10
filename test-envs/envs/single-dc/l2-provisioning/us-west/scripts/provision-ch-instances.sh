
set -e
# pass root vault token in to access all secrets
[ -n "$VAULT_TOKEN" ] || { echo VAULT_TOKEN environment variable is required; exit 7; }
export CHUSER=default
export ADMIN_PASSWORD=$( vault kv get -field=admin_password epl/clickhouse/testch )
export CH_URL="http://$CHUSER:$ADMIN_PASSWORD@10.17.0.11:8121"
while ! curl -s $CH_URL/?query=SELECT+1277712 | grep 1277712
do
    echo Waiting for clickhouse deployment testch to be up...
    sleep 5
done
echo Provisioning users and databases...
echo "CREATE DATABASE IF NOT EXISTS chdb_a ON CLUSTER default" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
export DB_USER_PASSWORD_HASH=$( vault kv get -field=db_chdb_a_admin epl/clickhouse/testch | sha256sum | awk '{print $1}' )
export DB_USER_NAME='db_chdb_a_admin'
echo "CREATE USER IF NOT EXISTS $DB_USER_NAME IDENTIFIED WITH sha256_hash BY '$DB_USER_PASSWORD_HASH'" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
export DB_USER_PASSWORD_HASH=$( vault kv get -field=db_chdb_a_rw epl/clickhouse/testch | sha256sum | awk '{print $1}' )
export DB_USER_NAME='db_chdb_a_rw'
echo "CREATE USER IF NOT EXISTS $DB_USER_NAME IDENTIFIED WITH sha256_hash BY '$DB_USER_PASSWORD_HASH'" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
export DB_USER_PASSWORD_HASH=$( vault kv get -field=db_chdb_a_ro epl/clickhouse/testch | sha256sum | awk '{print $1}' )
export DB_USER_NAME='db_chdb_a_ro'
echo "CREATE USER IF NOT EXISTS $DB_USER_NAME IDENTIFIED WITH sha256_hash BY '$DB_USER_PASSWORD_HASH'" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
echo "GRANT SELECT, SHOW ON chdb_a.* TO db_chdb_a_admin" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
echo "GRANT SELECT, SHOW ON chdb_a.* TO db_chdb_a_ro" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
echo "GRANT SELECT, SHOW ON chdb_a.* TO db_chdb_a_rw" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
echo "GRANT INSERT, OPTIMIZE ON chdb_a.* TO db_chdb_a_admin" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
echo "GRANT INSERT, OPTIMIZE ON chdb_a.* TO db_chdb_a_rw" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
echo "GRANT ALTER TABLE, ALTER VIEW, CREATE TABLE, CREATE VIEW, DROP TABLE, DROP VIEW, TRUNCATE ON chdb_a.* TO db_chdb_a_admin" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
echo "GRANT SOURCES, CLUSTER ON *.* TO db_chdb_a_admin" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
echo "GRANT TABLE ENGINE ON * TO db_chdb_a_admin" | \
  curl --data-binary @- -s --fail-with-body $CH_URL
echo Performing table migrations...
TARGET_DB_PASSWORD=$( vault kv get -field=db_chdb_a_admin epl/clickhouse/testch )
CH_DB_URL="http://db_chdb_a_admin:$TARGET_DB_PASSWORD@10.17.0.11:8121/?database=chdb_a" ch-migrations/up_testch.sh &
echo Migrations scheduled, waiting for finish...
wait
echo Provisioning NATS consumers...
echo "CREATE TABLE IF NOT EXISTS nats_ch_imp_queue_stream_import ( some_field Int64, some_text String ) ENGINE = NATS settings nats_url = 'epl-nats-main-nats.service.consul:4222', nats_queue_group = 'ch_imp_stream_import', nats_subjects = 'ch_imp.testch.chdb_a.stream_import', nats_format = 'JSONEachRow' " | \
  curl --data-binary @- -s --fail-with-body $CH_URL/?database=chdb_a
echo "CREATE MATERIALIZED VIEW IF NOT EXISTS nats_consumer_stream_import TO imp_table AS SELECT * FROM nats_ch_imp_queue_stream_import " | \
  curl --data-binary @- -s --fail-with-body $CH_URL/?database=chdb_a
wait
echo All migrations ran successfully
