#!/bin/sh
set -e

[ -n "$CH_DB_URL" ] || { echo CH_DB_URL environment variable is required; exit 7; }

echo "CREATE TABLE IF NOT EXISTS epl_schema_migrations ON CLUSTER default (logical_time Int64, time_started DateTime DEFAULT now() ) ENGINE = ReplicatedReplacingMergeTree ORDER BY logical_time" | \
  curl --data-binary @- -s --fail-with-body $CH_DB_URL

if ! echo "SELECT logical_time,'already_executed' FROM epl_schema_migrations WHERE logical_time = 1000" | curl --data-binary @- -s --fail-with-body $CH_DB_URL | grep already_executed
then
  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'WatUpWitItVanillaFace'

CREATE TABLE IF NOT EXISTS foo ON CLUSTER default (
            id Int64,
            a String,
            b String EPHEMERAL 'abc',
            c String DEFAULT upper(b),
            d String ALIAS lower(c),
            e String MATERIALIZED concat(id, a),
            f String DEFAULT 321
          ) ENGINE = ReplicatedMergeTree() ORDER BY id
WatUpWitItVanillaFace
  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'WatUpWitItVanillaFace'
  INSERT INTO epl_schema_migrations(logical_time) VALUES(1000)

WatUpWitItVanillaFace
fi

if ! echo "SELECT logical_time,'already_executed' FROM epl_schema_migrations WHERE logical_time = 2000" | curl --data-binary @- -s --fail-with-body $CH_DB_URL | grep already_executed
then
  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'WatUpWitItVanillaFace'

CREATE TABLE IF NOT EXISTS bar ON CLUSTER default (
            id Int64,
            b Bool
          ) engine = ReplicatedMergeTree() ORDER BY id
WatUpWitItVanillaFace
  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'WatUpWitItVanillaFace'
  INSERT INTO epl_schema_migrations(logical_time) VALUES(2000)

WatUpWitItVanillaFace
fi

if ! echo "SELECT logical_time,'already_executed' FROM epl_schema_migrations WHERE logical_time = 3000" | curl --data-binary @- -s --fail-with-body $CH_DB_URL | grep already_executed
then
  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'WatUpWitItVanillaFace'

CREATE TABLE IF NOT EXISTS imp_table ON CLUSTER default (
            some_field Int64,
            some_text String
          ) engine = ReplicatedMergeTree() ORDER BY some_field
WatUpWitItVanillaFace
  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'WatUpWitItVanillaFace'
  INSERT INTO epl_schema_migrations(logical_time) VALUES(3000)

WatUpWitItVanillaFace
fi

if ! echo "SELECT logical_time,'already_executed' FROM epl_schema_migrations WHERE logical_time = 4000" | curl --data-binary @- -s --fail-with-body $CH_DB_URL | grep already_executed
then
  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'WatUpWitItVanillaFace'

CREATE TABLE IF NOT EXISTS foo_ids ON CLUSTER default (
            id Int64
          ) engine = ReplicatedMergeTree() ORDER BY id
WatUpWitItVanillaFace
  curl --data-binary @- -s --fail-with-body $CH_DB_URL <<'WatUpWitItVanillaFace'
  INSERT INTO epl_schema_migrations(logical_time) VALUES(4000)

WatUpWitItVanillaFace
fi

