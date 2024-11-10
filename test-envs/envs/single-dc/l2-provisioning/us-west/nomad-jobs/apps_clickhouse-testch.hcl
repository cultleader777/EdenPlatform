job "clickhouse-testch" {
  type = "service"
  namespace = "apps"
  region = "us-west"
  datacenters = ["dc1"]

  vault {
    policies = ["epl-clickhouse-testch"]
  }
  update {
    auto_revert = false
    max_parallel = 1
    health_check = "checks"
    min_healthy_time = "30s"
    stagger = "30s"
    healthy_deadline = "300s"
    progress_deadline = "600s"
  }

  group "ch-1" {
    count = 1
    shutdown_delay = "0s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-clickhouse-server-b-testch}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "http_port" {
        static = 8121
        host_network = "lan"
      }
      port "native_port" {
        static = 8120
        host_network = "lan"
      }
      port "prom_port" {
        static = 8123
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "clickhouse"
      read_only = false
    }

    service {
      name = "epl-clickhouse-testch"
      port = "prom_port"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "prom_port"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "ch-testch-1" {
      driver = "docker"
      resources {
        memory = 448
        memory_max = 576
      }
      config {
        image = "clickhouse/clickhouse-server@sha256:2e6587b81a267c6152cf2112c3532516424d3eaa36f1b150d5b8847c0e3d5b01"
        network_mode = "host"
        entrypoint = [
          "/local/init",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/clickhouse"
      }

      template {
        destination = "secrets/clickhouse_config.xml"
        perms = "644"
        data = <<EOL

<clickhouse>
    <logger>
        <level>information</level>
        <console>true</console>
    </logger>

    <listen_host>10.17.0.11</listen_host>
    <http_port>8121</http_port>
    <tcp_port>8120</tcp_port>
    <interserver_http_port>8122</interserver_http_port>
    <interserver_http_host>10.17.0.11</interserver_http_host>
    <!-- decrease idle CPU usage https://github.com/ClickHouse/ClickHouse/issues/60016 -->
    <asynchronous_metrics_update_period_s>60</asynchronous_metrics_update_period_s>

    <prometheus>
        <endpoint>/metrics</endpoint>
        <port>8123</port>
        <metrics>true</metrics>
        <events>true</events>
        <asynchronous_metrics>true</asynchronous_metrics>
    </prometheus>

    <user_directories>
        <users_xml>
            <path>/secrets/users_config.xml</path>
        </users_xml>
        <replicated>
            <zookeeper_path>/ch-testch/access/</zookeeper_path>
        </replicated>
    </user_directories>

    <zookeeper>
        <sessions_path>/ch-testch/sessions</sessions_path>

            <node>
                <host>10.17.0.10</host>
                <port>9181</port>
            </node>

            <node>
                <host>10.17.0.11</host>
                <port>9181</port>
            </node>

            <node>
                <host>10.17.0.13</host>
                <port>9181</port>
            </node>

    </zookeeper>

    <macros>
        <shard>01</shard>
        <replica>testch-01-1</replica>
    </macros>

    <default_replica_path>/ch-testch/tables/{database}/{table}</default_replica_path>
    <default_replica_name>{replica}</default_replica_name>

    <remote_servers>
        <default>
            <shard>
                <!-- Optional. Whether to write data to just one of the replicas. Default: false (write data to all replicas). -->
                <internal_replication>true</internal_replication>

                <replica>
                    <host>10.17.0.11</host>
                    <port>8120</port>
                </replica>

                <replica>
                    <host>10.17.0.12</host>
                    <port>8120</port>
                </replica>

                <replica>
                    <host>10.17.0.13</host>
                    <port>8120</port>
                </replica>

            </shard>
        </default>
    </remote_servers>

    <!-- You can specify credentials for authenthication between replicas.
         This is required when interserver_https_port is accessible from untrusted networks,
         and also recommended to avoid SSRF attacks from possibly compromised services in your network.
      -->
    <interserver_http_credentials>
        <user>interserver</user>
        <password>{{ with secret "epl/data/clickhouse/testch" }}{{ .Data.data.interserver_password }}{{ end }}</password>
    </interserver_http_credentials>

    <max_connections>4096</max_connections>

    <!-- For 'Connection: keep-alive' in HTTP 1.1 -->
    <keep_alive_timeout>10</keep_alive_timeout>

    <!-- The maximum number of query processing threads, excluding threads for retrieving data from remote servers, allowed to run all queries.
         This is not a hard limit. In case if the limit is reached the query will still get at least one thread to run.
         Query can upscale to desired number of threads during execution if more threads become available.
    -->
    <concurrent_threads_soft_limit_num>0</concurrent_threads_soft_limit_num>
    <concurrent_threads_soft_limit_ratio_to_cores>2</concurrent_threads_soft_limit_ratio_to_cores>

    <!-- Maximum number of concurrent queries. -->
    <max_concurrent_queries>8</max_concurrent_queries>

    <!-- Maximum memory usage (resident set size) for server process.
         Zero value or unset means default. Default is "max_server_memory_usage_to_ram_ratio" of available physical RAM.
         If the value is larger than "max_server_memory_usage_to_ram_ratio" of available physical RAM, it will be cut down.

         The constraint is checked on query execution time.
         If a query tries to allocate memory and the current memory usage plus allocation is greater
          than specified threshold, exception will be thrown.

         It is not practical to set this constraint to small values like just a few gigabytes,
          because memory allocator will keep this amount of memory in caches and the server will deny service of queries.
      -->
    <max_server_memory_usage>436207616</max_server_memory_usage>

    <!-- Maximum number of threads in the Global thread pool.
    This will default to a maximum of 10000 threads if not specified.
    This setting will be useful in scenarios where there are a large number
    of distributed queries that are running concurrently but are idling most
    of the time, in which case a higher number of threads might be required.
    -->

    <max_thread_pool_size>1000</max_thread_pool_size>

    <!-- Configure other thread pools: -->

    <!-- Enables asynchronous loading of databases and tables to speedup server startup.
         Queries to not yet loaded entity will be blocked until load is finished.
      -->
    <!-- <async_load_databases>true</async_load_databases> -->

    <!-- On memory constrained environments you may have to set this to value larger than 1.
      -->
    <max_server_memory_usage_to_ram_ratio>0.9</max_server_memory_usage_to_ram_ratio>

    <!-- Simple server-wide memory profiler. Collect a stack trace at every peak allocation step (in bytes).
         Data will be stored in system.trace_log table with query_id = empty string.
         Zero means disabled.
      -->
    <total_memory_profiler_step>0</total_memory_profiler_step>

    <!-- Collect random allocations and deallocations and write them into system.trace_log with 'MemorySample' trace_type.
         The probability is for every alloc/free regardless to the size of the allocation.
         Note that sampling happens only when the amount of untracked memory exceeds the untracked memory limit,
          which is 4 MiB by default but can be lowered if 'total_memory_profiler_step' is lowered.
         You may want to set 'total_memory_profiler_step' to 1 for extra fine grained sampling.
      -->
    <total_memory_tracker_sample_probability>0</total_memory_tracker_sample_probability>

    <!-- Set limit on number of open files (default: maximum). This setting makes sense on Mac OS X because getrlimit() fails to retrieve
         correct maximum value. -->
    <!-- <max_open_files>262144</max_open_files> -->

    <!-- Size of cache of uncompressed blocks of data, used in tables of MergeTree family.
         In bytes. Cache is single for server. Memory is allocated only on demand.
         Cache is used when 'use_uncompressed_cache' user setting turned on (off by default).
         Uncompressed cache is advantageous only for very short queries and in rare cases.

         Note: uncompressed cache can be pointless for lz4, because memory bandwidth
         is slower than multi-core decompression on some server configurations.
         Enabling it can sometimes paradoxically make queries slower.
      -->
    <uncompressed_cache_size>268435456</uncompressed_cache_size>

    <!-- Approximate size of mark cache, used in tables of MergeTree family.
         In bytes. Cache is single for server. Memory is allocated only on demand.
         You should not lower this value.
      -->
    <mark_cache_size>134217728</mark_cache_size>

    <!-- For marks of secondary indices.
      -->
    <index_mark_cache_size>16777216</index_mark_cache_size>

    <!-- If you enable the `min_bytes_to_use_mmap_io` setting,
         the data in MergeTree tables can be read with mmap to avoid copying from kernel to userspace.
         It makes sense only for large files and helps only if data reside in page cache.
         To avoid frequent open/mmap/munmap/close calls (which are very expensive due to consequent page faults)
         and to reuse mappings from several threads and queries,
         the cache of mapped files is maintained. Its size is the number of mapped regions (usually equal to the number of mapped files).
         The amount of data in mapped files can be monitored
         in system.metrics, system.metric_log by the MMappedFiles, MMappedFileBytes metrics
         and in system.asynchronous_metrics, system.asynchronous_metrics_log by the MMapCacheCells metric,
         and also in system.events, system.processes, system.query_log, system.query_thread_log, system.query_views_log by the
         CreatedReadBufferMMap, CreatedReadBufferMMapFailed, MMappedFileCacheHits, MMappedFileCacheMisses events.
         Note that the amount of data in mapped files does not consume memory directly and is not accounted
         in query or server memory usage - because this memory can be discarded similar to OS page cache.
         The cache is dropped (the files are closed) automatically on removal of old parts in MergeTree,
         also it can be dropped manually by the SYSTEM DROP MMAP CACHE query.
      -->
    <mmap_cache_size>1000</mmap_cache_size>

    <!-- Cache size in bytes for compiled expressions.-->
    <compiled_expression_cache_size>8388608</compiled_expression_cache_size>

    <!-- Cache size in elements for compiled expressions.-->
    <compiled_expression_cache_elements_size>10000</compiled_expression_cache_elements_size>

    <!-- Cache path for custom (created from SQL) cached disks -->
    <custom_cached_disks_base_directory>/var/lib/clickhouse/caches/</custom_cached_disks_base_directory>

    <validate_tcp_client_information>false</validate_tcp_client_information>

    <!-- Path to data directory, with trailing slash. -->
    <path>/var/lib/clickhouse/</path>

    <!-- Path to temporary data for processing hard queries. -->
    <tmp_path>/var/lib/clickhouse/tmp/</tmp_path>

    <!-- Disable AuthType plaintext_password and no_password for ACL. -->
    <allow_plaintext_password>1</allow_plaintext_password>
    <allow_no_password>1</allow_no_password>
    <allow_implicit_no_password>1</allow_implicit_no_password>

    <!-- When a user does not specify a password type in the CREATE USER query, the default password type is used.
         Accepted values are: 'plaintext_password', 'sha256_password', 'double_sha1_password', 'bcrypt_password'.
      -->
    <default_password_type>sha256_password</default_password_type>

    <!-- Work factor for bcrypt_password authentication type-->
    <bcrypt_workfactor>12</bcrypt_workfactor>

    <!-- Directory with user provided files that are accessible by 'file' table function. -->
    <user_files_path>/var/lib/clickhouse/user_files/</user_files_path>

    <!-- Default profile of settings. -->
    <default_profile>default</default_profile>

    <!-- Comma-separated list of prefixes for user-defined settings.
         The server will allow to set these settings, and retrieve them with the getSetting function.
         They are also logged in the query_log, similarly to other settings, but have no special effect.
         The "SQL_" prefix is introduced for compatibility with MySQL - these settings are being set by Tableau.
    -->
    <custom_settings_prefixes>SQL_</custom_settings_prefixes>
    <default_database>default</default_database>

    <timezone>UTC</timezone>

    <!-- You can specify umask here (see "man umask"). Server will apply it on startup.
         Number is always parsed as octal. Default umask is 027 (other users cannot read logs, data files, etc; group can only read).
    -->
    <!-- <umask>022</umask> -->

    <!-- Perform mlockall after startup to lower first queries latency
          and to prevent clickhouse executable from being paged out under high IO load.
         Enabling this option is recommended but will lead to increased startup time for up to a few seconds.
    -->
    <mlock_executable>true</mlock_executable>

    <!-- Reallocate memory for machine code ("text") using huge pages. Highly experimental. -->
    <remap_executable>false</remap_executable>

    <!-- Substitutions for parameters of replicated tables.
          Optional. If you don't use replicated tables, you could omit that.

         See https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/replication/#creating-replicated-tables
      -->
    <!-- Replica group name for database Replicated.
          The cluster created by Replicated database will consist of replicas in the same group.
          DDL queries will only wail for the replicas in the same group.
          Empty by default.
    -->
    <!--
    <replica_group_name><replica_group_name>
    -->


    <!-- Reloading interval for embedded dictionaries, in seconds. Default: 3600. -->
    <builtin_dictionaries_reload_interval>3600</builtin_dictionaries_reload_interval>


    <!-- Maximum session timeout, in seconds. Default: 3600. -->
    <max_session_timeout>3600</max_session_timeout>

    <!-- Default session timeout, in seconds. Default: 60. -->
    <default_session_timeout>60</default_session_timeout>

    <!-- Sending data to Graphite for monitoring. Several sections can be defined. -->
    <!--
        interval - send every X second
        root_path - prefix for keys
        hostname_in_path - append hostname to root_path (default = true)
        metrics - send data from table system.metrics
        events - send data from table system.events
        asynchronous_metrics - send data from table system.asynchronous_metrics
    -->

    <!-- Serve endpoint for Prometheus monitoring. -->
    <!--
        endpoint - mertics path (relative to root, statring with "/")
        port - port to setup server. If not defined or 0 than http_port used
        metrics - send data from table system.metrics
        events - send data from table system.events
        asynchronous_metrics - send data from table system.asynchronous_metrics
    -->

    <!-- Query log. Used only for queries with setting log_queries = 1. -->
    <query_log>
        <!-- What table to insert data. If table is not exist, it will be created.
             When query log structure is changed after system update,
              then old table will be renamed and new table will be created automatically.
        -->
        <database>system</database>
        <table>query_log</table>
        <!--
            PARTITION BY expr: https://clickhouse.com/docs/en/table_engines/mergetree-family/custom_partitioning_key/
            Example:
                event_date
                toMonday(event_date)
                toYYYYMM(event_date)
                toStartOfHour(event_time)
        -->
        <partition_by>toYYYYMM(event_date)</partition_by>
        <!--
            Table TTL specification: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree/#mergetree-table-ttl
            Example:
                event_date + INTERVAL 1 WEEK
                event_date + INTERVAL 7 DAY DELETE
                event_date + INTERVAL 2 WEEK TO DISK 'bbb'

        <ttl>event_date + INTERVAL 30 DAY DELETE</ttl>
        -->

        <!--
            ORDER BY expr: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree#order_by
            Example:
                event_date, event_time
                event_date, type, query_id
                event_date, event_time, initial_query_id

        <order_by>event_date, event_time, initial_query_id</order_by>
        -->

        <!-- Instead of partition_by, you can provide full engine expression (starting with ENGINE = ) with parameters,
             Example: <engine>ENGINE = MergeTree PARTITION BY toYYYYMM(event_date) ORDER BY (event_date, event_time) SETTINGS index_granularity = 1024</engine>
          -->

        <!-- Interval of flushing data. -->
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <!-- Maximal size in lines for the logs. When non-flushed logs amount reaches max_size, logs dumped to the disk. -->
        <max_size_rows>1048576</max_size_rows>
        <!-- Pre-allocated size in lines for the logs. -->
        <reserved_size_rows>8192</reserved_size_rows>
        <!-- Lines amount threshold, reaching it launches flushing logs to the disk in background. -->
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <!-- Indication whether logs should be dumped to the disk in case of a crash -->
        <flush_on_crash>false</flush_on_crash>

        <!-- example of using a different storage policy for a system table -->
        <!-- storage_policy>local_ssd</storage_policy -->
    </query_log>

    <!-- Trace log. Stores stack traces collected by query profilers.
         See query_profiler_real_time_period_ns and query_profiler_cpu_time_period_ns settings. -->
    <trace_log>
        <database>system</database>
        <table>trace_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <!-- Indication whether logs should be dumped to the disk in case of a crash -->
        <flush_on_crash>false</flush_on_crash>
    </trace_log>

    <!-- Query thread log. Has information about all threads participated in query execution.
         Used only for queries with setting log_query_threads = 1. -->
    <query_thread_log>
        <database>system</database>
        <table>query_thread_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </query_thread_log>

    <!-- Query views log. Has information about all dependent views associated with a query.
         Used only for queries with setting log_query_views = 1. -->
    <query_views_log>
        <database>system</database>
        <table>query_views_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </query_views_log>

    <!-- Uncomment if use part log.
         Part log contains information about all actions with parts in MergeTree tables (creation, deletion, merges, downloads).-->
    <part_log>
        <database>system</database>
        <table>part_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </part_log>

    <!-- Uncomment to write text log into table.
         Text log contains all information from usual server log but stores it in structured and efficient way.
         The level of the messages that goes to the table can be limited (<level>), if not specified all messages will go to the table.
    <text_log>
        <database>system</database>
        <table>text_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
        <level></level>
    </text_log>
    -->

    <!-- Metric log contains rows with current values of ProfileEvents, CurrentMetrics collected with "collect_interval_milliseconds" interval. -->
    <metric_log>
        <database>system</database>
        <table>metric_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <collect_interval_milliseconds>1000</collect_interval_milliseconds>
        <flush_on_crash>false</flush_on_crash>
    </metric_log>

    <!--
        Asynchronous metric log contains values of metrics from
        system.asynchronous_metrics.
    -->
    <asynchronous_metric_log>
        <database>system</database>
        <table>asynchronous_metric_log</table>
        <flush_interval_milliseconds>7000</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </asynchronous_metric_log>

    <!--
        OpenTelemetry log contains OpenTelemetry trace spans.
    -->
    <opentelemetry_span_log>
        <!--
            The default table creation code is insufficient, this <engine> spec
            is a workaround. There is no 'event_time' for this log, but two times,
            start and finish. It is sorted by finish time, to avoid inserting
            data too far away in the past (probably we can sometimes insert a span
            that is seconds earlier than the last span in the table, due to a race
            between several spans inserted in parallel). This gives the spans a
            global order that we can use to e.g. retry insertion into some external
            system.
        -->
        <engine>
            engine MergeTree
            partition by toYYYYMM(finish_date)
            order by (finish_date, finish_time_us, trace_id)
        </engine>
        <database>system</database>
        <table>opentelemetry_span_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </opentelemetry_span_log>


    <!-- Crash log. Stores stack traces for fatal errors.
         This table is normally empty. -->
    <crash_log>
        <database>system</database>
        <table>crash_log</table>

        <partition_by />
        <flush_interval_milliseconds>1000</flush_interval_milliseconds>
        <max_size_rows>1024</max_size_rows>
        <reserved_size_rows>1024</reserved_size_rows>
        <buffer_size_rows_flush_threshold>512</buffer_size_rows_flush_threshold>
        <flush_on_crash>true</flush_on_crash>
    </crash_log>

    <!-- Session log. Stores user log in (successful or not) and log out events.

        Note: session log has known security issues and should not be used in production.
    -->
    <!-- <session_log>
        <database>system</database>
        <table>session_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </session_log> -->

    <!-- Profiling on Processors level. -->
    <processors_profile_log>
        <database>system</database>
        <table>processors_profile_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </processors_profile_log>

    <!-- Log of asynchronous inserts. It allows to check status
         of insert query in fire-and-forget mode.
    -->
    <asynchronous_insert_log>
        <database>system</database>
        <table>asynchronous_insert_log</table>

        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
        <partition_by>event_date</partition_by>
        <ttl>event_date + INTERVAL 3 DAY</ttl>
    </asynchronous_insert_log>

    <!-- Backup/restore log.
    -->
    <backup_log>
        <database>system</database>
        <table>backup_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </backup_log>

    <!-- Storage S3Queue log.
    -->
    <s3queue_log>
        <database>system</database>
        <table>s3queue_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </s3queue_log>

    <!-- Blob storage object operations log.
    -->
    <blob_storage_log>
        <database>system</database>
        <table>blob_storage_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <ttl>event_date + INTERVAL 30 DAY</ttl>
    </blob_storage_log>

    <!-- <top_level_domains_path>/var/lib/clickhouse/top_level_domains/</top_level_domains_path> -->
    <!-- Custom TLD lists.
         Format: <name>/path/to/file</name>

         Changes will not be applied w/o server restart.
         Path to the list is under top_level_domains_path (see above).
    -->
    <top_level_domains_lists>
        <!--
        <public_suffix_list>/path/to/public_suffix_list.dat</public_suffix_list>
        -->
    </top_level_domains_lists>

    <!-- Configuration of external dictionaries. See:
         https://clickhouse.com/docs/en/sql-reference/dictionaries/external-dictionaries/external-dicts
    -->
    <dictionaries_config>*_dictionary.*ml</dictionaries_config>

    <!-- Load dictionaries lazily, i.e. a dictionary will be loaded when it's used for the first time.
         "false" means ClickHouse will start loading dictionaries immediately at startup.
    -->
    <dictionaries_lazy_load>true</dictionaries_lazy_load>

    <!-- Wait at startup until all the dictionaries finish their loading (successfully or not)
         before receiving any connections. Affects dictionaries only if "dictionaries_lazy_load" is false.
         Setting this to false can make ClickHouse start faster, however some queries can be executed slower.
    -->
    <wait_dictionaries_load_at_startup>true</wait_dictionaries_load_at_startup>

    <!-- Configuration of user defined executable functions -->
    <user_defined_executable_functions_config>*_function.*ml</user_defined_executable_functions_config>

    <!-- Path in ZooKeeper to store user-defined SQL functions created by the command CREATE FUNCTION.
     If not specified they will be stored locally. -->
    <user_defined_zookeeper_path>/ch-testch/user_defined</user_defined_zookeeper_path>

    <!-- Uncomment if you want data to be compressed 30-100% better.
         Don't do that if you just started using ClickHouse.
      -->
    <!--
    <compression>
        <!- - Set of variants. Checked in order. Last matching case wins. If nothing matches, lz4 will be used. - ->
        <case>

            <!- - Conditions. All must be satisfied. Some conditions may be omitted. - ->
            <min_part_size>10000000000</min_part_size>        <!- - Min part size in bytes. - ->
            <min_part_size_ratio>0.01</min_part_size_ratio>   <!- - Min size of part relative to whole table size. - ->

            <!- - What compression method to use. - ->
            <method>zstd</method>
        </case>
    </compression>
    -->

    <!-- Allow to execute distributed DDL queries (CREATE, DROP, ALTER, RENAME) on cluster.
         Works only if ZooKeeper is enabled. Comment it if such functionality isn't required. -->
    <distributed_ddl>
        <!-- Path in ZooKeeper to queue with DDL queries -->
        <path>/ch-testch/task_queue/ddl</path>

        <!-- Settings from this profile will be used to execute DDL queries -->
        <!-- <profile>default</profile> -->

        <!-- Controls how much ON CLUSTER queries can be run simultaneously. -->
        <!-- <pool_size>1</pool_size> -->

        <!--
             Cleanup settings (active tasks will not be removed)
        -->

        <!-- Controls task TTL (default 1 week) -->
        <!-- <task_max_lifetime>604800</task_max_lifetime> -->

        <!-- Controls how often cleanup should be performed (in seconds) -->
        <!-- <cleanup_delay_period>60</cleanup_delay_period> -->

        <!-- Controls how many tasks could be in the queue -->
        <!-- <max_tasks_in_queue>1000</max_tasks_in_queue> -->

        <!-- Host name of the current node. If specified, will only compare and not resolve hostnames inside the DDL tasks -->
        <host_name>10.17.0.11</host_name>
    </distributed_ddl>

    <!-- Settings to fine-tune MergeTree tables. See documentation in source code, in MergeTreeSettings.h -->
    <merge_tree>
        <number_of_free_entries_in_pool_to_lower_max_size_of_merge>0</number_of_free_entries_in_pool_to_lower_max_size_of_merge>
        <!-- <max_suspicious_broken_parts>5</max_suspicious_broken_parts> -->
        <!--
        Choose a value between 1024 and 4096.
        The default is 8192.
        -->
        <merge_max_block_size>1024</merge_max_block_size>
        <max_bytes_to_merge_at_max_space_in_pool>1073741824</max_bytes_to_merge_at_max_space_in_pool>
    </merge_tree>

    <!-- Settings to fine-tune ReplicatedMergeTree tables. See documentation in source code, in MergeTreeSettings.h -->
    <!--
    <replicated_merge_tree>
        <max_replicated_fetches_network_bandwidth>1000000000</max_replicated_fetches_network_bandwidth>
    </replicated_merge_tree>
    -->

    <!-- Settings to fine-tune Distributed tables. See documentation in source code, in DistributedSettings.h -->
    <!--
    <distributed>
        <flush_on_detach>false</flush_on_detach>
    </distributed>
    -->

    <!-- Protection from accidental DROP.
         If size of a MergeTree table is greater than max_table_size_to_drop (in bytes) than table could not be dropped with any DROP query.
         If you want do delete one table and don't want to change clickhouse-server config, you could create special file <clickhouse-path>/flags/force_drop_table and make DROP once.
         By default max_table_size_to_drop is 50GB; max_table_size_to_drop=0 allows to DROP any tables.
         The same for max_partition_size_to_drop.
         Uncomment to disable protection.
    -->
    <!-- <max_table_size_to_drop>0</max_table_size_to_drop> -->
    <!-- <max_partition_size_to_drop>0</max_partition_size_to_drop> -->

    <!-- Example of parameters for GraphiteMergeTree table engine -->

    <!-- Directory in <clickhouse-path> containing schema files for various input formats.
         The directory will be created if it doesn't exist.
      -->
    <format_schema_path>/var/lib/clickhouse/format_schemas/</format_schema_path>

    <!-- Directory containing the proto files for the well-known Protobuf types.
      -->
    <google_protos_path>/usr/share/clickhouse/protos/</google_protos_path>

    <!-- Configuration for the query cache -->
    <query_cache>
        <max_size_in_bytes>8388608</max_size_in_bytes>
        <max_entries>1024</max_entries>
        <max_entry_size_in_bytes>1048576</max_entry_size_in_bytes>
        <max_entry_size_in_rows>30000000</max_entry_size_in_rows>
    </query_cache>

    <backups>
        <allowed_path>backups</allowed_path>

        <!-- If the BACKUP command fails and this setting is true then the files
             copied before the failure will be removed automatically.
        -->
        <remove_backup_files_after_failure>true</remove_backup_files_after_failure>
    </backups>

    <!-- This allows to disable exposing addresses in stack traces for security reasons.
         Please be aware that it does not improve security much, but makes debugging much harder.
         The addresses that are small offsets from zero will be displayed nevertheless to show nullptr dereferences.
         Regardless of this configuration, the addresses are visible in the system.stack_trace and system.trace_log tables
         if the user has access to these tables.
         I don't recommend to change this setting.
    <show_addresses_in_stack_traces>false</show_addresses_in_stack_traces>
    -->

</clickhouse>
EOL
      }

      template {
        destination = "secrets/env_vars"
        perms = "644"
        env = true
        data = <<EOL
CH_ADMIN_PASSWORD={{ with secret "epl/data/clickhouse/testch" }}{{ .Data.data.admin_password }}{{ end }}
EOL
      }

      template {
        destination = "secrets/users_config.xml"
        perms = "644"
        data = <<EOL

<clickhouse>
    <!-- Profiles of settings. -->
    <profiles>
        <!-- Default settings. -->
        <default>
        </default>

        <!-- Profile that allows only read queries. -->
        <readonly>
            <readonly>1</readonly>
        </readonly>
    </profiles>

    <!-- Users and ACL. -->
    <users>
        <!-- If user name was not specified, 'default' user is used. -->
        <default>
            <!-- See also the files in users.d directory where the password can be overridden.

                 Password could be specified in plaintext or in SHA256 (in hex format).

                 If you want to specify password in plaintext (not recommended), place it in 'password' element.
                 Example: <password>qwerty</password>.
                 Password could be empty.

                 If you want to specify SHA256, place it in 'password_sha256_hex' element.
                 Example: <password_sha256_hex>65e84be33532fb784c48129675f9eff3a682b27168c0ea744b2cf58ee02337c5</password_sha256_hex>
                 Restrictions of SHA256: impossibility to connect to ClickHouse using MySQL JS client (as of July 2019).

                 If you want to specify double SHA1, place it in 'password_double_sha1_hex' element.
                 Example: <password_double_sha1_hex>e395796d6546b1b65db9d665cd43f0e858dd4303</password_double_sha1_hex>

                 If you want to specify a previously defined LDAP server (see 'ldap_servers' in the main config) for authentication,
                  place its name in 'server' element inside 'ldap' element.
                 Example: <ldap><server>my_ldap_server</server></ldap>

                 If you want to authenticate the user via Kerberos (assuming Kerberos is enabled, see 'kerberos' in the main config),
                  place 'kerberos' element instead of 'password' (and similar) elements.
                 The name part of the canonical principal name of the initiator must match the user name for authentication to succeed.
                 You can also place 'realm' element inside 'kerberos' element to further restrict authentication to only those requests
                  whose initiator's realm matches it.
                 Example: <kerberos />
                 Example: <kerberos><realm>EXAMPLE.COM</realm></kerberos>

                 How to generate decent password:
                 Execute: PASSWORD=$(base64 < /dev/urandom | head -c8); echo "$PASSWORD"; echo -n "$PASSWORD" | sha256sum | tr -d '-'
                 In first line will be password and in second - corresponding SHA256.

                 How to generate double SHA1:
                 Execute: PASSWORD=$(base64 < /dev/urandom | head -c8); echo "$PASSWORD"; echo -n "$PASSWORD" | sha1sum | tr -d '-' | xxd -r -p | sha1sum | tr -d '-'
                 In first line will be password and in second - corresponding double SHA1.
            -->
            <password>{{ with secret "epl/data/clickhouse/testch" }}{{ .Data.data.admin_password }}{{ end }}</password>

            <!-- List of networks with open access.

                 To open access from everywhere, specify:
                    <ip>::/0</ip>

                 To open access only from localhost, specify:
                    <ip>::1</ip>
                    <ip>127.0.0.1</ip>

                 Each element of list has one of the following forms:
                 <ip> IP-address or network mask. Examples: 213.180.204.3 or 10.0.0.1/8 or 10.0.0.1/255.255.255.0
                     2a02:6b8::3 or 2a02:6b8::3/64 or 2a02:6b8::3/ffff:ffff:ffff:ffff::.
                 <host> Hostname. Example: server01.clickhouse.com.
                     To check access, DNS query is performed, and all received addresses compared to peer address.
                 <host_regexp> Regular expression for host names. Example, ^server\d\d-\d\d-\d\.clickhouse\.com$
                     To check access, DNS PTR query is performed for peer address and then regexp is applied.
                     Then, for result of PTR query, another DNS query is performed and all received addresses compared to peer address.
                     Strongly recommended that regexp is ends with $
                 All results of DNS requests are cached till server restart.
            -->
            <networks>
                <!-- eden platform subnet -->
                <ip>10.0.0.0/8</ip>
            </networks>

            <!-- Settings profile for user. -->
            <profile>default</profile>

            <!-- Quota for user. -->
            <quota>default</quota>

            <!-- User can create other users and grant rights to them. -->
            <access_management>1</access_management>

            <!-- User can manipulate named collections. -->
            <named_collection_control>1</named_collection_control>

            <!-- User permissions can be granted here -->
            <!--
            <grants>
                <query>GRANT ALL ON *.*</query>
            </grants>
            -->
        </default>
    </users>

    <!-- Quotas. -->
    <quotas>
        <!-- Name of quota. -->
        <default>
            <!-- Limits for time interval. You could specify many intervals with different limits. -->
            <interval>
                <!-- Length of interval. -->
                <duration>3600</duration>

                <!-- No limits. Just calculate resource usage for time interval. -->
                <queries>0</queries>
                <errors>0</errors>
                <result_rows>0</result_rows>
                <read_rows>0</read_rows>
                <execution_time>0</execution_time>
                <queue_max_wait_ms>1000</queue_max_wait_ms>
                <max_execution_time>10</max_execution_time>
            </interval>
        </default>
    </quotas>
</clickhouse>
EOL
      }

      template {
        destination = "local/init"
        perms = "755"
        data = <<EOL
#!/bin/sh
# helper executable
echo '#!/bin/sh

 clickhouse-client -h 10.17.0.11 --port 8120 --password $CH_ADMIN_PASSWORD
' > /usr/local/bin/connect
chmod +x /usr/local/bin/connect

exec /usr/bin/clickhouse-server --config-file=/secrets/clickhouse_config.xml
EOL
      }
    }
  }

  group "ch-2" {
    count = 1
    shutdown_delay = "30s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-clickhouse-server-c-testch}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "http_port" {
        static = 8121
        host_network = "lan"
      }
      port "native_port" {
        static = 8120
        host_network = "lan"
      }
      port "prom_port" {
        static = 8123
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "clickhouse"
      read_only = false
    }

    service {
      name = "epl-clickhouse-testch"
      port = "prom_port"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "prom_port"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "ch-testch-2" {
      driver = "docker"
      resources {
        memory = 448
        memory_max = 576
      }
      config {
        image = "clickhouse/clickhouse-server@sha256:2e6587b81a267c6152cf2112c3532516424d3eaa36f1b150d5b8847c0e3d5b01"
        network_mode = "host"
        entrypoint = [
          "/local/init",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/clickhouse"
      }

      template {
        destination = "secrets/clickhouse_config.xml"
        perms = "644"
        data = <<EOL

<clickhouse>
    <logger>
        <level>information</level>
        <console>true</console>
    </logger>

    <listen_host>10.17.0.12</listen_host>
    <http_port>8121</http_port>
    <tcp_port>8120</tcp_port>
    <interserver_http_port>8122</interserver_http_port>
    <interserver_http_host>10.17.0.12</interserver_http_host>
    <!-- decrease idle CPU usage https://github.com/ClickHouse/ClickHouse/issues/60016 -->
    <asynchronous_metrics_update_period_s>60</asynchronous_metrics_update_period_s>

    <prometheus>
        <endpoint>/metrics</endpoint>
        <port>8123</port>
        <metrics>true</metrics>
        <events>true</events>
        <asynchronous_metrics>true</asynchronous_metrics>
    </prometheus>

    <user_directories>
        <users_xml>
            <path>/secrets/users_config.xml</path>
        </users_xml>
        <replicated>
            <zookeeper_path>/ch-testch/access/</zookeeper_path>
        </replicated>
    </user_directories>

    <zookeeper>
        <sessions_path>/ch-testch/sessions</sessions_path>

            <node>
                <host>10.17.0.10</host>
                <port>9181</port>
            </node>

            <node>
                <host>10.17.0.11</host>
                <port>9181</port>
            </node>

            <node>
                <host>10.17.0.13</host>
                <port>9181</port>
            </node>

    </zookeeper>

    <macros>
        <shard>01</shard>
        <replica>testch-01-2</replica>
    </macros>

    <default_replica_path>/ch-testch/tables/{database}/{table}</default_replica_path>
    <default_replica_name>{replica}</default_replica_name>

    <remote_servers>
        <default>
            <shard>
                <!-- Optional. Whether to write data to just one of the replicas. Default: false (write data to all replicas). -->
                <internal_replication>true</internal_replication>

                <replica>
                    <host>10.17.0.11</host>
                    <port>8120</port>
                </replica>

                <replica>
                    <host>10.17.0.12</host>
                    <port>8120</port>
                </replica>

                <replica>
                    <host>10.17.0.13</host>
                    <port>8120</port>
                </replica>

            </shard>
        </default>
    </remote_servers>

    <!-- You can specify credentials for authenthication between replicas.
         This is required when interserver_https_port is accessible from untrusted networks,
         and also recommended to avoid SSRF attacks from possibly compromised services in your network.
      -->
    <interserver_http_credentials>
        <user>interserver</user>
        <password>{{ with secret "epl/data/clickhouse/testch" }}{{ .Data.data.interserver_password }}{{ end }}</password>
    </interserver_http_credentials>

    <max_connections>4096</max_connections>

    <!-- For 'Connection: keep-alive' in HTTP 1.1 -->
    <keep_alive_timeout>10</keep_alive_timeout>

    <!-- The maximum number of query processing threads, excluding threads for retrieving data from remote servers, allowed to run all queries.
         This is not a hard limit. In case if the limit is reached the query will still get at least one thread to run.
         Query can upscale to desired number of threads during execution if more threads become available.
    -->
    <concurrent_threads_soft_limit_num>0</concurrent_threads_soft_limit_num>
    <concurrent_threads_soft_limit_ratio_to_cores>2</concurrent_threads_soft_limit_ratio_to_cores>

    <!-- Maximum number of concurrent queries. -->
    <max_concurrent_queries>8</max_concurrent_queries>

    <!-- Maximum memory usage (resident set size) for server process.
         Zero value or unset means default. Default is "max_server_memory_usage_to_ram_ratio" of available physical RAM.
         If the value is larger than "max_server_memory_usage_to_ram_ratio" of available physical RAM, it will be cut down.

         The constraint is checked on query execution time.
         If a query tries to allocate memory and the current memory usage plus allocation is greater
          than specified threshold, exception will be thrown.

         It is not practical to set this constraint to small values like just a few gigabytes,
          because memory allocator will keep this amount of memory in caches and the server will deny service of queries.
      -->
    <max_server_memory_usage>436207616</max_server_memory_usage>

    <!-- Maximum number of threads in the Global thread pool.
    This will default to a maximum of 10000 threads if not specified.
    This setting will be useful in scenarios where there are a large number
    of distributed queries that are running concurrently but are idling most
    of the time, in which case a higher number of threads might be required.
    -->

    <max_thread_pool_size>1000</max_thread_pool_size>

    <!-- Configure other thread pools: -->

    <!-- Enables asynchronous loading of databases and tables to speedup server startup.
         Queries to not yet loaded entity will be blocked until load is finished.
      -->
    <!-- <async_load_databases>true</async_load_databases> -->

    <!-- On memory constrained environments you may have to set this to value larger than 1.
      -->
    <max_server_memory_usage_to_ram_ratio>0.9</max_server_memory_usage_to_ram_ratio>

    <!-- Simple server-wide memory profiler. Collect a stack trace at every peak allocation step (in bytes).
         Data will be stored in system.trace_log table with query_id = empty string.
         Zero means disabled.
      -->
    <total_memory_profiler_step>0</total_memory_profiler_step>

    <!-- Collect random allocations and deallocations and write them into system.trace_log with 'MemorySample' trace_type.
         The probability is for every alloc/free regardless to the size of the allocation.
         Note that sampling happens only when the amount of untracked memory exceeds the untracked memory limit,
          which is 4 MiB by default but can be lowered if 'total_memory_profiler_step' is lowered.
         You may want to set 'total_memory_profiler_step' to 1 for extra fine grained sampling.
      -->
    <total_memory_tracker_sample_probability>0</total_memory_tracker_sample_probability>

    <!-- Set limit on number of open files (default: maximum). This setting makes sense on Mac OS X because getrlimit() fails to retrieve
         correct maximum value. -->
    <!-- <max_open_files>262144</max_open_files> -->

    <!-- Size of cache of uncompressed blocks of data, used in tables of MergeTree family.
         In bytes. Cache is single for server. Memory is allocated only on demand.
         Cache is used when 'use_uncompressed_cache' user setting turned on (off by default).
         Uncompressed cache is advantageous only for very short queries and in rare cases.

         Note: uncompressed cache can be pointless for lz4, because memory bandwidth
         is slower than multi-core decompression on some server configurations.
         Enabling it can sometimes paradoxically make queries slower.
      -->
    <uncompressed_cache_size>268435456</uncompressed_cache_size>

    <!-- Approximate size of mark cache, used in tables of MergeTree family.
         In bytes. Cache is single for server. Memory is allocated only on demand.
         You should not lower this value.
      -->
    <mark_cache_size>134217728</mark_cache_size>

    <!-- For marks of secondary indices.
      -->
    <index_mark_cache_size>16777216</index_mark_cache_size>

    <!-- If you enable the `min_bytes_to_use_mmap_io` setting,
         the data in MergeTree tables can be read with mmap to avoid copying from kernel to userspace.
         It makes sense only for large files and helps only if data reside in page cache.
         To avoid frequent open/mmap/munmap/close calls (which are very expensive due to consequent page faults)
         and to reuse mappings from several threads and queries,
         the cache of mapped files is maintained. Its size is the number of mapped regions (usually equal to the number of mapped files).
         The amount of data in mapped files can be monitored
         in system.metrics, system.metric_log by the MMappedFiles, MMappedFileBytes metrics
         and in system.asynchronous_metrics, system.asynchronous_metrics_log by the MMapCacheCells metric,
         and also in system.events, system.processes, system.query_log, system.query_thread_log, system.query_views_log by the
         CreatedReadBufferMMap, CreatedReadBufferMMapFailed, MMappedFileCacheHits, MMappedFileCacheMisses events.
         Note that the amount of data in mapped files does not consume memory directly and is not accounted
         in query or server memory usage - because this memory can be discarded similar to OS page cache.
         The cache is dropped (the files are closed) automatically on removal of old parts in MergeTree,
         also it can be dropped manually by the SYSTEM DROP MMAP CACHE query.
      -->
    <mmap_cache_size>1000</mmap_cache_size>

    <!-- Cache size in bytes for compiled expressions.-->
    <compiled_expression_cache_size>8388608</compiled_expression_cache_size>

    <!-- Cache size in elements for compiled expressions.-->
    <compiled_expression_cache_elements_size>10000</compiled_expression_cache_elements_size>

    <!-- Cache path for custom (created from SQL) cached disks -->
    <custom_cached_disks_base_directory>/var/lib/clickhouse/caches/</custom_cached_disks_base_directory>

    <validate_tcp_client_information>false</validate_tcp_client_information>

    <!-- Path to data directory, with trailing slash. -->
    <path>/var/lib/clickhouse/</path>

    <!-- Path to temporary data for processing hard queries. -->
    <tmp_path>/var/lib/clickhouse/tmp/</tmp_path>

    <!-- Disable AuthType plaintext_password and no_password for ACL. -->
    <allow_plaintext_password>1</allow_plaintext_password>
    <allow_no_password>1</allow_no_password>
    <allow_implicit_no_password>1</allow_implicit_no_password>

    <!-- When a user does not specify a password type in the CREATE USER query, the default password type is used.
         Accepted values are: 'plaintext_password', 'sha256_password', 'double_sha1_password', 'bcrypt_password'.
      -->
    <default_password_type>sha256_password</default_password_type>

    <!-- Work factor for bcrypt_password authentication type-->
    <bcrypt_workfactor>12</bcrypt_workfactor>

    <!-- Directory with user provided files that are accessible by 'file' table function. -->
    <user_files_path>/var/lib/clickhouse/user_files/</user_files_path>

    <!-- Default profile of settings. -->
    <default_profile>default</default_profile>

    <!-- Comma-separated list of prefixes for user-defined settings.
         The server will allow to set these settings, and retrieve them with the getSetting function.
         They are also logged in the query_log, similarly to other settings, but have no special effect.
         The "SQL_" prefix is introduced for compatibility with MySQL - these settings are being set by Tableau.
    -->
    <custom_settings_prefixes>SQL_</custom_settings_prefixes>
    <default_database>default</default_database>

    <timezone>UTC</timezone>

    <!-- You can specify umask here (see "man umask"). Server will apply it on startup.
         Number is always parsed as octal. Default umask is 027 (other users cannot read logs, data files, etc; group can only read).
    -->
    <!-- <umask>022</umask> -->

    <!-- Perform mlockall after startup to lower first queries latency
          and to prevent clickhouse executable from being paged out under high IO load.
         Enabling this option is recommended but will lead to increased startup time for up to a few seconds.
    -->
    <mlock_executable>true</mlock_executable>

    <!-- Reallocate memory for machine code ("text") using huge pages. Highly experimental. -->
    <remap_executable>false</remap_executable>

    <!-- Substitutions for parameters of replicated tables.
          Optional. If you don't use replicated tables, you could omit that.

         See https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/replication/#creating-replicated-tables
      -->
    <!-- Replica group name for database Replicated.
          The cluster created by Replicated database will consist of replicas in the same group.
          DDL queries will only wail for the replicas in the same group.
          Empty by default.
    -->
    <!--
    <replica_group_name><replica_group_name>
    -->


    <!-- Reloading interval for embedded dictionaries, in seconds. Default: 3600. -->
    <builtin_dictionaries_reload_interval>3600</builtin_dictionaries_reload_interval>


    <!-- Maximum session timeout, in seconds. Default: 3600. -->
    <max_session_timeout>3600</max_session_timeout>

    <!-- Default session timeout, in seconds. Default: 60. -->
    <default_session_timeout>60</default_session_timeout>

    <!-- Sending data to Graphite for monitoring. Several sections can be defined. -->
    <!--
        interval - send every X second
        root_path - prefix for keys
        hostname_in_path - append hostname to root_path (default = true)
        metrics - send data from table system.metrics
        events - send data from table system.events
        asynchronous_metrics - send data from table system.asynchronous_metrics
    -->

    <!-- Serve endpoint for Prometheus monitoring. -->
    <!--
        endpoint - mertics path (relative to root, statring with "/")
        port - port to setup server. If not defined or 0 than http_port used
        metrics - send data from table system.metrics
        events - send data from table system.events
        asynchronous_metrics - send data from table system.asynchronous_metrics
    -->

    <!-- Query log. Used only for queries with setting log_queries = 1. -->
    <query_log>
        <!-- What table to insert data. If table is not exist, it will be created.
             When query log structure is changed after system update,
              then old table will be renamed and new table will be created automatically.
        -->
        <database>system</database>
        <table>query_log</table>
        <!--
            PARTITION BY expr: https://clickhouse.com/docs/en/table_engines/mergetree-family/custom_partitioning_key/
            Example:
                event_date
                toMonday(event_date)
                toYYYYMM(event_date)
                toStartOfHour(event_time)
        -->
        <partition_by>toYYYYMM(event_date)</partition_by>
        <!--
            Table TTL specification: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree/#mergetree-table-ttl
            Example:
                event_date + INTERVAL 1 WEEK
                event_date + INTERVAL 7 DAY DELETE
                event_date + INTERVAL 2 WEEK TO DISK 'bbb'

        <ttl>event_date + INTERVAL 30 DAY DELETE</ttl>
        -->

        <!--
            ORDER BY expr: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree#order_by
            Example:
                event_date, event_time
                event_date, type, query_id
                event_date, event_time, initial_query_id

        <order_by>event_date, event_time, initial_query_id</order_by>
        -->

        <!-- Instead of partition_by, you can provide full engine expression (starting with ENGINE = ) with parameters,
             Example: <engine>ENGINE = MergeTree PARTITION BY toYYYYMM(event_date) ORDER BY (event_date, event_time) SETTINGS index_granularity = 1024</engine>
          -->

        <!-- Interval of flushing data. -->
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <!-- Maximal size in lines for the logs. When non-flushed logs amount reaches max_size, logs dumped to the disk. -->
        <max_size_rows>1048576</max_size_rows>
        <!-- Pre-allocated size in lines for the logs. -->
        <reserved_size_rows>8192</reserved_size_rows>
        <!-- Lines amount threshold, reaching it launches flushing logs to the disk in background. -->
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <!-- Indication whether logs should be dumped to the disk in case of a crash -->
        <flush_on_crash>false</flush_on_crash>

        <!-- example of using a different storage policy for a system table -->
        <!-- storage_policy>local_ssd</storage_policy -->
    </query_log>

    <!-- Trace log. Stores stack traces collected by query profilers.
         See query_profiler_real_time_period_ns and query_profiler_cpu_time_period_ns settings. -->
    <trace_log>
        <database>system</database>
        <table>trace_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <!-- Indication whether logs should be dumped to the disk in case of a crash -->
        <flush_on_crash>false</flush_on_crash>
    </trace_log>

    <!-- Query thread log. Has information about all threads participated in query execution.
         Used only for queries with setting log_query_threads = 1. -->
    <query_thread_log>
        <database>system</database>
        <table>query_thread_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </query_thread_log>

    <!-- Query views log. Has information about all dependent views associated with a query.
         Used only for queries with setting log_query_views = 1. -->
    <query_views_log>
        <database>system</database>
        <table>query_views_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </query_views_log>

    <!-- Uncomment if use part log.
         Part log contains information about all actions with parts in MergeTree tables (creation, deletion, merges, downloads).-->
    <part_log>
        <database>system</database>
        <table>part_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </part_log>

    <!-- Uncomment to write text log into table.
         Text log contains all information from usual server log but stores it in structured and efficient way.
         The level of the messages that goes to the table can be limited (<level>), if not specified all messages will go to the table.
    <text_log>
        <database>system</database>
        <table>text_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
        <level></level>
    </text_log>
    -->

    <!-- Metric log contains rows with current values of ProfileEvents, CurrentMetrics collected with "collect_interval_milliseconds" interval. -->
    <metric_log>
        <database>system</database>
        <table>metric_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <collect_interval_milliseconds>1000</collect_interval_milliseconds>
        <flush_on_crash>false</flush_on_crash>
    </metric_log>

    <!--
        Asynchronous metric log contains values of metrics from
        system.asynchronous_metrics.
    -->
    <asynchronous_metric_log>
        <database>system</database>
        <table>asynchronous_metric_log</table>
        <flush_interval_milliseconds>7000</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </asynchronous_metric_log>

    <!--
        OpenTelemetry log contains OpenTelemetry trace spans.
    -->
    <opentelemetry_span_log>
        <!--
            The default table creation code is insufficient, this <engine> spec
            is a workaround. There is no 'event_time' for this log, but two times,
            start and finish. It is sorted by finish time, to avoid inserting
            data too far away in the past (probably we can sometimes insert a span
            that is seconds earlier than the last span in the table, due to a race
            between several spans inserted in parallel). This gives the spans a
            global order that we can use to e.g. retry insertion into some external
            system.
        -->
        <engine>
            engine MergeTree
            partition by toYYYYMM(finish_date)
            order by (finish_date, finish_time_us, trace_id)
        </engine>
        <database>system</database>
        <table>opentelemetry_span_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </opentelemetry_span_log>


    <!-- Crash log. Stores stack traces for fatal errors.
         This table is normally empty. -->
    <crash_log>
        <database>system</database>
        <table>crash_log</table>

        <partition_by />
        <flush_interval_milliseconds>1000</flush_interval_milliseconds>
        <max_size_rows>1024</max_size_rows>
        <reserved_size_rows>1024</reserved_size_rows>
        <buffer_size_rows_flush_threshold>512</buffer_size_rows_flush_threshold>
        <flush_on_crash>true</flush_on_crash>
    </crash_log>

    <!-- Session log. Stores user log in (successful or not) and log out events.

        Note: session log has known security issues and should not be used in production.
    -->
    <!-- <session_log>
        <database>system</database>
        <table>session_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </session_log> -->

    <!-- Profiling on Processors level. -->
    <processors_profile_log>
        <database>system</database>
        <table>processors_profile_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </processors_profile_log>

    <!-- Log of asynchronous inserts. It allows to check status
         of insert query in fire-and-forget mode.
    -->
    <asynchronous_insert_log>
        <database>system</database>
        <table>asynchronous_insert_log</table>

        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
        <partition_by>event_date</partition_by>
        <ttl>event_date + INTERVAL 3 DAY</ttl>
    </asynchronous_insert_log>

    <!-- Backup/restore log.
    -->
    <backup_log>
        <database>system</database>
        <table>backup_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </backup_log>

    <!-- Storage S3Queue log.
    -->
    <s3queue_log>
        <database>system</database>
        <table>s3queue_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </s3queue_log>

    <!-- Blob storage object operations log.
    -->
    <blob_storage_log>
        <database>system</database>
        <table>blob_storage_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <ttl>event_date + INTERVAL 30 DAY</ttl>
    </blob_storage_log>

    <!-- <top_level_domains_path>/var/lib/clickhouse/top_level_domains/</top_level_domains_path> -->
    <!-- Custom TLD lists.
         Format: <name>/path/to/file</name>

         Changes will not be applied w/o server restart.
         Path to the list is under top_level_domains_path (see above).
    -->
    <top_level_domains_lists>
        <!--
        <public_suffix_list>/path/to/public_suffix_list.dat</public_suffix_list>
        -->
    </top_level_domains_lists>

    <!-- Configuration of external dictionaries. See:
         https://clickhouse.com/docs/en/sql-reference/dictionaries/external-dictionaries/external-dicts
    -->
    <dictionaries_config>*_dictionary.*ml</dictionaries_config>

    <!-- Load dictionaries lazily, i.e. a dictionary will be loaded when it's used for the first time.
         "false" means ClickHouse will start loading dictionaries immediately at startup.
    -->
    <dictionaries_lazy_load>true</dictionaries_lazy_load>

    <!-- Wait at startup until all the dictionaries finish their loading (successfully or not)
         before receiving any connections. Affects dictionaries only if "dictionaries_lazy_load" is false.
         Setting this to false can make ClickHouse start faster, however some queries can be executed slower.
    -->
    <wait_dictionaries_load_at_startup>true</wait_dictionaries_load_at_startup>

    <!-- Configuration of user defined executable functions -->
    <user_defined_executable_functions_config>*_function.*ml</user_defined_executable_functions_config>

    <!-- Path in ZooKeeper to store user-defined SQL functions created by the command CREATE FUNCTION.
     If not specified they will be stored locally. -->
    <user_defined_zookeeper_path>/ch-testch/user_defined</user_defined_zookeeper_path>

    <!-- Uncomment if you want data to be compressed 30-100% better.
         Don't do that if you just started using ClickHouse.
      -->
    <!--
    <compression>
        <!- - Set of variants. Checked in order. Last matching case wins. If nothing matches, lz4 will be used. - ->
        <case>

            <!- - Conditions. All must be satisfied. Some conditions may be omitted. - ->
            <min_part_size>10000000000</min_part_size>        <!- - Min part size in bytes. - ->
            <min_part_size_ratio>0.01</min_part_size_ratio>   <!- - Min size of part relative to whole table size. - ->

            <!- - What compression method to use. - ->
            <method>zstd</method>
        </case>
    </compression>
    -->

    <!-- Allow to execute distributed DDL queries (CREATE, DROP, ALTER, RENAME) on cluster.
         Works only if ZooKeeper is enabled. Comment it if such functionality isn't required. -->
    <distributed_ddl>
        <!-- Path in ZooKeeper to queue with DDL queries -->
        <path>/ch-testch/task_queue/ddl</path>

        <!-- Settings from this profile will be used to execute DDL queries -->
        <!-- <profile>default</profile> -->

        <!-- Controls how much ON CLUSTER queries can be run simultaneously. -->
        <!-- <pool_size>1</pool_size> -->

        <!--
             Cleanup settings (active tasks will not be removed)
        -->

        <!-- Controls task TTL (default 1 week) -->
        <!-- <task_max_lifetime>604800</task_max_lifetime> -->

        <!-- Controls how often cleanup should be performed (in seconds) -->
        <!-- <cleanup_delay_period>60</cleanup_delay_period> -->

        <!-- Controls how many tasks could be in the queue -->
        <!-- <max_tasks_in_queue>1000</max_tasks_in_queue> -->

        <!-- Host name of the current node. If specified, will only compare and not resolve hostnames inside the DDL tasks -->
        <host_name>10.17.0.12</host_name>
    </distributed_ddl>

    <!-- Settings to fine-tune MergeTree tables. See documentation in source code, in MergeTreeSettings.h -->
    <merge_tree>
        <number_of_free_entries_in_pool_to_lower_max_size_of_merge>0</number_of_free_entries_in_pool_to_lower_max_size_of_merge>
        <!-- <max_suspicious_broken_parts>5</max_suspicious_broken_parts> -->
        <!--
        Choose a value between 1024 and 4096.
        The default is 8192.
        -->
        <merge_max_block_size>1024</merge_max_block_size>
        <max_bytes_to_merge_at_max_space_in_pool>1073741824</max_bytes_to_merge_at_max_space_in_pool>
    </merge_tree>

    <!-- Settings to fine-tune ReplicatedMergeTree tables. See documentation in source code, in MergeTreeSettings.h -->
    <!--
    <replicated_merge_tree>
        <max_replicated_fetches_network_bandwidth>1000000000</max_replicated_fetches_network_bandwidth>
    </replicated_merge_tree>
    -->

    <!-- Settings to fine-tune Distributed tables. See documentation in source code, in DistributedSettings.h -->
    <!--
    <distributed>
        <flush_on_detach>false</flush_on_detach>
    </distributed>
    -->

    <!-- Protection from accidental DROP.
         If size of a MergeTree table is greater than max_table_size_to_drop (in bytes) than table could not be dropped with any DROP query.
         If you want do delete one table and don't want to change clickhouse-server config, you could create special file <clickhouse-path>/flags/force_drop_table and make DROP once.
         By default max_table_size_to_drop is 50GB; max_table_size_to_drop=0 allows to DROP any tables.
         The same for max_partition_size_to_drop.
         Uncomment to disable protection.
    -->
    <!-- <max_table_size_to_drop>0</max_table_size_to_drop> -->
    <!-- <max_partition_size_to_drop>0</max_partition_size_to_drop> -->

    <!-- Example of parameters for GraphiteMergeTree table engine -->

    <!-- Directory in <clickhouse-path> containing schema files for various input formats.
         The directory will be created if it doesn't exist.
      -->
    <format_schema_path>/var/lib/clickhouse/format_schemas/</format_schema_path>

    <!-- Directory containing the proto files for the well-known Protobuf types.
      -->
    <google_protos_path>/usr/share/clickhouse/protos/</google_protos_path>

    <!-- Configuration for the query cache -->
    <query_cache>
        <max_size_in_bytes>8388608</max_size_in_bytes>
        <max_entries>1024</max_entries>
        <max_entry_size_in_bytes>1048576</max_entry_size_in_bytes>
        <max_entry_size_in_rows>30000000</max_entry_size_in_rows>
    </query_cache>

    <backups>
        <allowed_path>backups</allowed_path>

        <!-- If the BACKUP command fails and this setting is true then the files
             copied before the failure will be removed automatically.
        -->
        <remove_backup_files_after_failure>true</remove_backup_files_after_failure>
    </backups>

    <!-- This allows to disable exposing addresses in stack traces for security reasons.
         Please be aware that it does not improve security much, but makes debugging much harder.
         The addresses that are small offsets from zero will be displayed nevertheless to show nullptr dereferences.
         Regardless of this configuration, the addresses are visible in the system.stack_trace and system.trace_log tables
         if the user has access to these tables.
         I don't recommend to change this setting.
    <show_addresses_in_stack_traces>false</show_addresses_in_stack_traces>
    -->

</clickhouse>
EOL
      }

      template {
        destination = "secrets/env_vars"
        perms = "644"
        env = true
        data = <<EOL
CH_ADMIN_PASSWORD={{ with secret "epl/data/clickhouse/testch" }}{{ .Data.data.admin_password }}{{ end }}
EOL
      }

      template {
        destination = "secrets/users_config.xml"
        perms = "644"
        data = <<EOL

<clickhouse>
    <!-- Profiles of settings. -->
    <profiles>
        <!-- Default settings. -->
        <default>
        </default>

        <!-- Profile that allows only read queries. -->
        <readonly>
            <readonly>1</readonly>
        </readonly>
    </profiles>

    <!-- Users and ACL. -->
    <users>
        <!-- If user name was not specified, 'default' user is used. -->
        <default>
            <!-- See also the files in users.d directory where the password can be overridden.

                 Password could be specified in plaintext or in SHA256 (in hex format).

                 If you want to specify password in plaintext (not recommended), place it in 'password' element.
                 Example: <password>qwerty</password>.
                 Password could be empty.

                 If you want to specify SHA256, place it in 'password_sha256_hex' element.
                 Example: <password_sha256_hex>65e84be33532fb784c48129675f9eff3a682b27168c0ea744b2cf58ee02337c5</password_sha256_hex>
                 Restrictions of SHA256: impossibility to connect to ClickHouse using MySQL JS client (as of July 2019).

                 If you want to specify double SHA1, place it in 'password_double_sha1_hex' element.
                 Example: <password_double_sha1_hex>e395796d6546b1b65db9d665cd43f0e858dd4303</password_double_sha1_hex>

                 If you want to specify a previously defined LDAP server (see 'ldap_servers' in the main config) for authentication,
                  place its name in 'server' element inside 'ldap' element.
                 Example: <ldap><server>my_ldap_server</server></ldap>

                 If you want to authenticate the user via Kerberos (assuming Kerberos is enabled, see 'kerberos' in the main config),
                  place 'kerberos' element instead of 'password' (and similar) elements.
                 The name part of the canonical principal name of the initiator must match the user name for authentication to succeed.
                 You can also place 'realm' element inside 'kerberos' element to further restrict authentication to only those requests
                  whose initiator's realm matches it.
                 Example: <kerberos />
                 Example: <kerberos><realm>EXAMPLE.COM</realm></kerberos>

                 How to generate decent password:
                 Execute: PASSWORD=$(base64 < /dev/urandom | head -c8); echo "$PASSWORD"; echo -n "$PASSWORD" | sha256sum | tr -d '-'
                 In first line will be password and in second - corresponding SHA256.

                 How to generate double SHA1:
                 Execute: PASSWORD=$(base64 < /dev/urandom | head -c8); echo "$PASSWORD"; echo -n "$PASSWORD" | sha1sum | tr -d '-' | xxd -r -p | sha1sum | tr -d '-'
                 In first line will be password and in second - corresponding double SHA1.
            -->
            <password>{{ with secret "epl/data/clickhouse/testch" }}{{ .Data.data.admin_password }}{{ end }}</password>

            <!-- List of networks with open access.

                 To open access from everywhere, specify:
                    <ip>::/0</ip>

                 To open access only from localhost, specify:
                    <ip>::1</ip>
                    <ip>127.0.0.1</ip>

                 Each element of list has one of the following forms:
                 <ip> IP-address or network mask. Examples: 213.180.204.3 or 10.0.0.1/8 or 10.0.0.1/255.255.255.0
                     2a02:6b8::3 or 2a02:6b8::3/64 or 2a02:6b8::3/ffff:ffff:ffff:ffff::.
                 <host> Hostname. Example: server01.clickhouse.com.
                     To check access, DNS query is performed, and all received addresses compared to peer address.
                 <host_regexp> Regular expression for host names. Example, ^server\d\d-\d\d-\d\.clickhouse\.com$
                     To check access, DNS PTR query is performed for peer address and then regexp is applied.
                     Then, for result of PTR query, another DNS query is performed and all received addresses compared to peer address.
                     Strongly recommended that regexp is ends with $
                 All results of DNS requests are cached till server restart.
            -->
            <networks>
                <!-- eden platform subnet -->
                <ip>10.0.0.0/8</ip>
            </networks>

            <!-- Settings profile for user. -->
            <profile>default</profile>

            <!-- Quota for user. -->
            <quota>default</quota>

            <!-- User can create other users and grant rights to them. -->
            <access_management>1</access_management>

            <!-- User can manipulate named collections. -->
            <named_collection_control>1</named_collection_control>

            <!-- User permissions can be granted here -->
            <!--
            <grants>
                <query>GRANT ALL ON *.*</query>
            </grants>
            -->
        </default>
    </users>

    <!-- Quotas. -->
    <quotas>
        <!-- Name of quota. -->
        <default>
            <!-- Limits for time interval. You could specify many intervals with different limits. -->
            <interval>
                <!-- Length of interval. -->
                <duration>3600</duration>

                <!-- No limits. Just calculate resource usage for time interval. -->
                <queries>0</queries>
                <errors>0</errors>
                <result_rows>0</result_rows>
                <read_rows>0</read_rows>
                <execution_time>0</execution_time>
                <queue_max_wait_ms>1000</queue_max_wait_ms>
                <max_execution_time>10</max_execution_time>
            </interval>
        </default>
    </quotas>
</clickhouse>
EOL
      }

      template {
        destination = "local/init"
        perms = "755"
        data = <<EOL
#!/bin/sh
# helper executable
echo '#!/bin/sh

 clickhouse-client -h 10.17.0.12 --port 8120 --password $CH_ADMIN_PASSWORD
' > /usr/local/bin/connect
chmod +x /usr/local/bin/connect

exec /usr/bin/clickhouse-server --config-file=/secrets/clickhouse_config.xml
EOL
      }
    }
  }

  group "ch-3" {
    count = 1
    shutdown_delay = "60s"

    constraint {
        attribute = "${attr.kernel.arch}"
        value     = "x86_64"
    }
    constraint {
      attribute = "${meta.lock_epl-clickhouse-server-d-testch}"
      operator  = ">"
      value     = "0"
    }
    network {
      mode = "host"
      port "http_port" {
        static = 8121
        host_network = "lan"
      }
      port "native_port" {
        static = 8120
        host_network = "lan"
      }
      port "prom_port" {
        static = 8123
        host_network = "lan"
      }
    }

    volume "v_1" {
      type = "host"
      source = "clickhouse"
      read_only = false
    }

    service {
      name = "epl-clickhouse-testch"
      port = "prom_port"
      address = "${meta.private_ip}"
      tags = ["epl-mon-default"]
      meta {
        metrics_path = "/metrics"
      }
      check {
        type = "tcp"
        port = "prom_port"
        interval = "10s"
        timeout = "2s"
      }
    }

    task "ch-testch-3" {
      driver = "docker"
      resources {
        memory = 448
        memory_max = 576
      }
      config {
        image = "clickhouse/clickhouse-server@sha256:2e6587b81a267c6152cf2112c3532516424d3eaa36f1b150d5b8847c0e3d5b01"
        network_mode = "host"
        entrypoint = [
          "/local/init",
        ]
        labels {
          epl_loki_cluster = "main"
        }
      }

      volume_mount {
        volume = "v_1"
        destination = "/var/lib/clickhouse"
      }

      template {
        destination = "secrets/clickhouse_config.xml"
        perms = "644"
        data = <<EOL

<clickhouse>
    <logger>
        <level>information</level>
        <console>true</console>
    </logger>

    <listen_host>10.17.0.13</listen_host>
    <http_port>8121</http_port>
    <tcp_port>8120</tcp_port>
    <interserver_http_port>8122</interserver_http_port>
    <interserver_http_host>10.17.0.13</interserver_http_host>
    <!-- decrease idle CPU usage https://github.com/ClickHouse/ClickHouse/issues/60016 -->
    <asynchronous_metrics_update_period_s>60</asynchronous_metrics_update_period_s>

    <prometheus>
        <endpoint>/metrics</endpoint>
        <port>8123</port>
        <metrics>true</metrics>
        <events>true</events>
        <asynchronous_metrics>true</asynchronous_metrics>
    </prometheus>

    <user_directories>
        <users_xml>
            <path>/secrets/users_config.xml</path>
        </users_xml>
        <replicated>
            <zookeeper_path>/ch-testch/access/</zookeeper_path>
        </replicated>
    </user_directories>

    <zookeeper>
        <sessions_path>/ch-testch/sessions</sessions_path>

            <node>
                <host>10.17.0.10</host>
                <port>9181</port>
            </node>

            <node>
                <host>10.17.0.11</host>
                <port>9181</port>
            </node>

            <node>
                <host>10.17.0.13</host>
                <port>9181</port>
            </node>

    </zookeeper>

    <macros>
        <shard>01</shard>
        <replica>testch-01-3</replica>
    </macros>

    <default_replica_path>/ch-testch/tables/{database}/{table}</default_replica_path>
    <default_replica_name>{replica}</default_replica_name>

    <remote_servers>
        <default>
            <shard>
                <!-- Optional. Whether to write data to just one of the replicas. Default: false (write data to all replicas). -->
                <internal_replication>true</internal_replication>

                <replica>
                    <host>10.17.0.11</host>
                    <port>8120</port>
                </replica>

                <replica>
                    <host>10.17.0.12</host>
                    <port>8120</port>
                </replica>

                <replica>
                    <host>10.17.0.13</host>
                    <port>8120</port>
                </replica>

            </shard>
        </default>
    </remote_servers>

    <!-- You can specify credentials for authenthication between replicas.
         This is required when interserver_https_port is accessible from untrusted networks,
         and also recommended to avoid SSRF attacks from possibly compromised services in your network.
      -->
    <interserver_http_credentials>
        <user>interserver</user>
        <password>{{ with secret "epl/data/clickhouse/testch" }}{{ .Data.data.interserver_password }}{{ end }}</password>
    </interserver_http_credentials>

    <max_connections>4096</max_connections>

    <!-- For 'Connection: keep-alive' in HTTP 1.1 -->
    <keep_alive_timeout>10</keep_alive_timeout>

    <!-- The maximum number of query processing threads, excluding threads for retrieving data from remote servers, allowed to run all queries.
         This is not a hard limit. In case if the limit is reached the query will still get at least one thread to run.
         Query can upscale to desired number of threads during execution if more threads become available.
    -->
    <concurrent_threads_soft_limit_num>0</concurrent_threads_soft_limit_num>
    <concurrent_threads_soft_limit_ratio_to_cores>2</concurrent_threads_soft_limit_ratio_to_cores>

    <!-- Maximum number of concurrent queries. -->
    <max_concurrent_queries>8</max_concurrent_queries>

    <!-- Maximum memory usage (resident set size) for server process.
         Zero value or unset means default. Default is "max_server_memory_usage_to_ram_ratio" of available physical RAM.
         If the value is larger than "max_server_memory_usage_to_ram_ratio" of available physical RAM, it will be cut down.

         The constraint is checked on query execution time.
         If a query tries to allocate memory and the current memory usage plus allocation is greater
          than specified threshold, exception will be thrown.

         It is not practical to set this constraint to small values like just a few gigabytes,
          because memory allocator will keep this amount of memory in caches and the server will deny service of queries.
      -->
    <max_server_memory_usage>436207616</max_server_memory_usage>

    <!-- Maximum number of threads in the Global thread pool.
    This will default to a maximum of 10000 threads if not specified.
    This setting will be useful in scenarios where there are a large number
    of distributed queries that are running concurrently but are idling most
    of the time, in which case a higher number of threads might be required.
    -->

    <max_thread_pool_size>1000</max_thread_pool_size>

    <!-- Configure other thread pools: -->

    <!-- Enables asynchronous loading of databases and tables to speedup server startup.
         Queries to not yet loaded entity will be blocked until load is finished.
      -->
    <!-- <async_load_databases>true</async_load_databases> -->

    <!-- On memory constrained environments you may have to set this to value larger than 1.
      -->
    <max_server_memory_usage_to_ram_ratio>0.9</max_server_memory_usage_to_ram_ratio>

    <!-- Simple server-wide memory profiler. Collect a stack trace at every peak allocation step (in bytes).
         Data will be stored in system.trace_log table with query_id = empty string.
         Zero means disabled.
      -->
    <total_memory_profiler_step>0</total_memory_profiler_step>

    <!-- Collect random allocations and deallocations and write them into system.trace_log with 'MemorySample' trace_type.
         The probability is for every alloc/free regardless to the size of the allocation.
         Note that sampling happens only when the amount of untracked memory exceeds the untracked memory limit,
          which is 4 MiB by default but can be lowered if 'total_memory_profiler_step' is lowered.
         You may want to set 'total_memory_profiler_step' to 1 for extra fine grained sampling.
      -->
    <total_memory_tracker_sample_probability>0</total_memory_tracker_sample_probability>

    <!-- Set limit on number of open files (default: maximum). This setting makes sense on Mac OS X because getrlimit() fails to retrieve
         correct maximum value. -->
    <!-- <max_open_files>262144</max_open_files> -->

    <!-- Size of cache of uncompressed blocks of data, used in tables of MergeTree family.
         In bytes. Cache is single for server. Memory is allocated only on demand.
         Cache is used when 'use_uncompressed_cache' user setting turned on (off by default).
         Uncompressed cache is advantageous only for very short queries and in rare cases.

         Note: uncompressed cache can be pointless for lz4, because memory bandwidth
         is slower than multi-core decompression on some server configurations.
         Enabling it can sometimes paradoxically make queries slower.
      -->
    <uncompressed_cache_size>268435456</uncompressed_cache_size>

    <!-- Approximate size of mark cache, used in tables of MergeTree family.
         In bytes. Cache is single for server. Memory is allocated only on demand.
         You should not lower this value.
      -->
    <mark_cache_size>134217728</mark_cache_size>

    <!-- For marks of secondary indices.
      -->
    <index_mark_cache_size>16777216</index_mark_cache_size>

    <!-- If you enable the `min_bytes_to_use_mmap_io` setting,
         the data in MergeTree tables can be read with mmap to avoid copying from kernel to userspace.
         It makes sense only for large files and helps only if data reside in page cache.
         To avoid frequent open/mmap/munmap/close calls (which are very expensive due to consequent page faults)
         and to reuse mappings from several threads and queries,
         the cache of mapped files is maintained. Its size is the number of mapped regions (usually equal to the number of mapped files).
         The amount of data in mapped files can be monitored
         in system.metrics, system.metric_log by the MMappedFiles, MMappedFileBytes metrics
         and in system.asynchronous_metrics, system.asynchronous_metrics_log by the MMapCacheCells metric,
         and also in system.events, system.processes, system.query_log, system.query_thread_log, system.query_views_log by the
         CreatedReadBufferMMap, CreatedReadBufferMMapFailed, MMappedFileCacheHits, MMappedFileCacheMisses events.
         Note that the amount of data in mapped files does not consume memory directly and is not accounted
         in query or server memory usage - because this memory can be discarded similar to OS page cache.
         The cache is dropped (the files are closed) automatically on removal of old parts in MergeTree,
         also it can be dropped manually by the SYSTEM DROP MMAP CACHE query.
      -->
    <mmap_cache_size>1000</mmap_cache_size>

    <!-- Cache size in bytes for compiled expressions.-->
    <compiled_expression_cache_size>8388608</compiled_expression_cache_size>

    <!-- Cache size in elements for compiled expressions.-->
    <compiled_expression_cache_elements_size>10000</compiled_expression_cache_elements_size>

    <!-- Cache path for custom (created from SQL) cached disks -->
    <custom_cached_disks_base_directory>/var/lib/clickhouse/caches/</custom_cached_disks_base_directory>

    <validate_tcp_client_information>false</validate_tcp_client_information>

    <!-- Path to data directory, with trailing slash. -->
    <path>/var/lib/clickhouse/</path>

    <!-- Path to temporary data for processing hard queries. -->
    <tmp_path>/var/lib/clickhouse/tmp/</tmp_path>

    <!-- Disable AuthType plaintext_password and no_password for ACL. -->
    <allow_plaintext_password>1</allow_plaintext_password>
    <allow_no_password>1</allow_no_password>
    <allow_implicit_no_password>1</allow_implicit_no_password>

    <!-- When a user does not specify a password type in the CREATE USER query, the default password type is used.
         Accepted values are: 'plaintext_password', 'sha256_password', 'double_sha1_password', 'bcrypt_password'.
      -->
    <default_password_type>sha256_password</default_password_type>

    <!-- Work factor for bcrypt_password authentication type-->
    <bcrypt_workfactor>12</bcrypt_workfactor>

    <!-- Directory with user provided files that are accessible by 'file' table function. -->
    <user_files_path>/var/lib/clickhouse/user_files/</user_files_path>

    <!-- Default profile of settings. -->
    <default_profile>default</default_profile>

    <!-- Comma-separated list of prefixes for user-defined settings.
         The server will allow to set these settings, and retrieve them with the getSetting function.
         They are also logged in the query_log, similarly to other settings, but have no special effect.
         The "SQL_" prefix is introduced for compatibility with MySQL - these settings are being set by Tableau.
    -->
    <custom_settings_prefixes>SQL_</custom_settings_prefixes>
    <default_database>default</default_database>

    <timezone>UTC</timezone>

    <!-- You can specify umask here (see "man umask"). Server will apply it on startup.
         Number is always parsed as octal. Default umask is 027 (other users cannot read logs, data files, etc; group can only read).
    -->
    <!-- <umask>022</umask> -->

    <!-- Perform mlockall after startup to lower first queries latency
          and to prevent clickhouse executable from being paged out under high IO load.
         Enabling this option is recommended but will lead to increased startup time for up to a few seconds.
    -->
    <mlock_executable>true</mlock_executable>

    <!-- Reallocate memory for machine code ("text") using huge pages. Highly experimental. -->
    <remap_executable>false</remap_executable>

    <!-- Substitutions for parameters of replicated tables.
          Optional. If you don't use replicated tables, you could omit that.

         See https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/replication/#creating-replicated-tables
      -->
    <!-- Replica group name for database Replicated.
          The cluster created by Replicated database will consist of replicas in the same group.
          DDL queries will only wail for the replicas in the same group.
          Empty by default.
    -->
    <!--
    <replica_group_name><replica_group_name>
    -->


    <!-- Reloading interval for embedded dictionaries, in seconds. Default: 3600. -->
    <builtin_dictionaries_reload_interval>3600</builtin_dictionaries_reload_interval>


    <!-- Maximum session timeout, in seconds. Default: 3600. -->
    <max_session_timeout>3600</max_session_timeout>

    <!-- Default session timeout, in seconds. Default: 60. -->
    <default_session_timeout>60</default_session_timeout>

    <!-- Sending data to Graphite for monitoring. Several sections can be defined. -->
    <!--
        interval - send every X second
        root_path - prefix for keys
        hostname_in_path - append hostname to root_path (default = true)
        metrics - send data from table system.metrics
        events - send data from table system.events
        asynchronous_metrics - send data from table system.asynchronous_metrics
    -->

    <!-- Serve endpoint for Prometheus monitoring. -->
    <!--
        endpoint - mertics path (relative to root, statring with "/")
        port - port to setup server. If not defined or 0 than http_port used
        metrics - send data from table system.metrics
        events - send data from table system.events
        asynchronous_metrics - send data from table system.asynchronous_metrics
    -->

    <!-- Query log. Used only for queries with setting log_queries = 1. -->
    <query_log>
        <!-- What table to insert data. If table is not exist, it will be created.
             When query log structure is changed after system update,
              then old table will be renamed and new table will be created automatically.
        -->
        <database>system</database>
        <table>query_log</table>
        <!--
            PARTITION BY expr: https://clickhouse.com/docs/en/table_engines/mergetree-family/custom_partitioning_key/
            Example:
                event_date
                toMonday(event_date)
                toYYYYMM(event_date)
                toStartOfHour(event_time)
        -->
        <partition_by>toYYYYMM(event_date)</partition_by>
        <!--
            Table TTL specification: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree/#mergetree-table-ttl
            Example:
                event_date + INTERVAL 1 WEEK
                event_date + INTERVAL 7 DAY DELETE
                event_date + INTERVAL 2 WEEK TO DISK 'bbb'

        <ttl>event_date + INTERVAL 30 DAY DELETE</ttl>
        -->

        <!--
            ORDER BY expr: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree#order_by
            Example:
                event_date, event_time
                event_date, type, query_id
                event_date, event_time, initial_query_id

        <order_by>event_date, event_time, initial_query_id</order_by>
        -->

        <!-- Instead of partition_by, you can provide full engine expression (starting with ENGINE = ) with parameters,
             Example: <engine>ENGINE = MergeTree PARTITION BY toYYYYMM(event_date) ORDER BY (event_date, event_time) SETTINGS index_granularity = 1024</engine>
          -->

        <!-- Interval of flushing data. -->
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <!-- Maximal size in lines for the logs. When non-flushed logs amount reaches max_size, logs dumped to the disk. -->
        <max_size_rows>1048576</max_size_rows>
        <!-- Pre-allocated size in lines for the logs. -->
        <reserved_size_rows>8192</reserved_size_rows>
        <!-- Lines amount threshold, reaching it launches flushing logs to the disk in background. -->
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <!-- Indication whether logs should be dumped to the disk in case of a crash -->
        <flush_on_crash>false</flush_on_crash>

        <!-- example of using a different storage policy for a system table -->
        <!-- storage_policy>local_ssd</storage_policy -->
    </query_log>

    <!-- Trace log. Stores stack traces collected by query profilers.
         See query_profiler_real_time_period_ns and query_profiler_cpu_time_period_ns settings. -->
    <trace_log>
        <database>system</database>
        <table>trace_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <!-- Indication whether logs should be dumped to the disk in case of a crash -->
        <flush_on_crash>false</flush_on_crash>
    </trace_log>

    <!-- Query thread log. Has information about all threads participated in query execution.
         Used only for queries with setting log_query_threads = 1. -->
    <query_thread_log>
        <database>system</database>
        <table>query_thread_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </query_thread_log>

    <!-- Query views log. Has information about all dependent views associated with a query.
         Used only for queries with setting log_query_views = 1. -->
    <query_views_log>
        <database>system</database>
        <table>query_views_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </query_views_log>

    <!-- Uncomment if use part log.
         Part log contains information about all actions with parts in MergeTree tables (creation, deletion, merges, downloads).-->
    <part_log>
        <database>system</database>
        <table>part_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </part_log>

    <!-- Uncomment to write text log into table.
         Text log contains all information from usual server log but stores it in structured and efficient way.
         The level of the messages that goes to the table can be limited (<level>), if not specified all messages will go to the table.
    <text_log>
        <database>system</database>
        <table>text_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
        <level></level>
    </text_log>
    -->

    <!-- Metric log contains rows with current values of ProfileEvents, CurrentMetrics collected with "collect_interval_milliseconds" interval. -->
    <metric_log>
        <database>system</database>
        <table>metric_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <collect_interval_milliseconds>1000</collect_interval_milliseconds>
        <flush_on_crash>false</flush_on_crash>
    </metric_log>

    <!--
        Asynchronous metric log contains values of metrics from
        system.asynchronous_metrics.
    -->
    <asynchronous_metric_log>
        <database>system</database>
        <table>asynchronous_metric_log</table>
        <flush_interval_milliseconds>7000</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </asynchronous_metric_log>

    <!--
        OpenTelemetry log contains OpenTelemetry trace spans.
    -->
    <opentelemetry_span_log>
        <!--
            The default table creation code is insufficient, this <engine> spec
            is a workaround. There is no 'event_time' for this log, but two times,
            start and finish. It is sorted by finish time, to avoid inserting
            data too far away in the past (probably we can sometimes insert a span
            that is seconds earlier than the last span in the table, due to a race
            between several spans inserted in parallel). This gives the spans a
            global order that we can use to e.g. retry insertion into some external
            system.
        -->
        <engine>
            engine MergeTree
            partition by toYYYYMM(finish_date)
            order by (finish_date, finish_time_us, trace_id)
        </engine>
        <database>system</database>
        <table>opentelemetry_span_log</table>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </opentelemetry_span_log>


    <!-- Crash log. Stores stack traces for fatal errors.
         This table is normally empty. -->
    <crash_log>
        <database>system</database>
        <table>crash_log</table>

        <partition_by />
        <flush_interval_milliseconds>1000</flush_interval_milliseconds>
        <max_size_rows>1024</max_size_rows>
        <reserved_size_rows>1024</reserved_size_rows>
        <buffer_size_rows_flush_threshold>512</buffer_size_rows_flush_threshold>
        <flush_on_crash>true</flush_on_crash>
    </crash_log>

    <!-- Session log. Stores user log in (successful or not) and log out events.

        Note: session log has known security issues and should not be used in production.
    -->
    <!-- <session_log>
        <database>system</database>
        <table>session_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </session_log> -->

    <!-- Profiling on Processors level. -->
    <processors_profile_log>
        <database>system</database>
        <table>processors_profile_log</table>

        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
    </processors_profile_log>

    <!-- Log of asynchronous inserts. It allows to check status
         of insert query in fire-and-forget mode.
    -->
    <asynchronous_insert_log>
        <database>system</database>
        <table>asynchronous_insert_log</table>

        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <max_size_rows>1048576</max_size_rows>
        <reserved_size_rows>8192</reserved_size_rows>
        <buffer_size_rows_flush_threshold>524288</buffer_size_rows_flush_threshold>
        <flush_on_crash>false</flush_on_crash>
        <partition_by>event_date</partition_by>
        <ttl>event_date + INTERVAL 3 DAY</ttl>
    </asynchronous_insert_log>

    <!-- Backup/restore log.
    -->
    <backup_log>
        <database>system</database>
        <table>backup_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </backup_log>

    <!-- Storage S3Queue log.
    -->
    <s3queue_log>
        <database>system</database>
        <table>s3queue_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
    </s3queue_log>

    <!-- Blob storage object operations log.
    -->
    <blob_storage_log>
        <database>system</database>
        <table>blob_storage_log</table>
        <partition_by>toYYYYMM(event_date)</partition_by>
        <flush_interval_milliseconds>7500</flush_interval_milliseconds>
        <ttl>event_date + INTERVAL 30 DAY</ttl>
    </blob_storage_log>

    <!-- <top_level_domains_path>/var/lib/clickhouse/top_level_domains/</top_level_domains_path> -->
    <!-- Custom TLD lists.
         Format: <name>/path/to/file</name>

         Changes will not be applied w/o server restart.
         Path to the list is under top_level_domains_path (see above).
    -->
    <top_level_domains_lists>
        <!--
        <public_suffix_list>/path/to/public_suffix_list.dat</public_suffix_list>
        -->
    </top_level_domains_lists>

    <!-- Configuration of external dictionaries. See:
         https://clickhouse.com/docs/en/sql-reference/dictionaries/external-dictionaries/external-dicts
    -->
    <dictionaries_config>*_dictionary.*ml</dictionaries_config>

    <!-- Load dictionaries lazily, i.e. a dictionary will be loaded when it's used for the first time.
         "false" means ClickHouse will start loading dictionaries immediately at startup.
    -->
    <dictionaries_lazy_load>true</dictionaries_lazy_load>

    <!-- Wait at startup until all the dictionaries finish their loading (successfully or not)
         before receiving any connections. Affects dictionaries only if "dictionaries_lazy_load" is false.
         Setting this to false can make ClickHouse start faster, however some queries can be executed slower.
    -->
    <wait_dictionaries_load_at_startup>true</wait_dictionaries_load_at_startup>

    <!-- Configuration of user defined executable functions -->
    <user_defined_executable_functions_config>*_function.*ml</user_defined_executable_functions_config>

    <!-- Path in ZooKeeper to store user-defined SQL functions created by the command CREATE FUNCTION.
     If not specified they will be stored locally. -->
    <user_defined_zookeeper_path>/ch-testch/user_defined</user_defined_zookeeper_path>

    <!-- Uncomment if you want data to be compressed 30-100% better.
         Don't do that if you just started using ClickHouse.
      -->
    <!--
    <compression>
        <!- - Set of variants. Checked in order. Last matching case wins. If nothing matches, lz4 will be used. - ->
        <case>

            <!- - Conditions. All must be satisfied. Some conditions may be omitted. - ->
            <min_part_size>10000000000</min_part_size>        <!- - Min part size in bytes. - ->
            <min_part_size_ratio>0.01</min_part_size_ratio>   <!- - Min size of part relative to whole table size. - ->

            <!- - What compression method to use. - ->
            <method>zstd</method>
        </case>
    </compression>
    -->

    <!-- Allow to execute distributed DDL queries (CREATE, DROP, ALTER, RENAME) on cluster.
         Works only if ZooKeeper is enabled. Comment it if such functionality isn't required. -->
    <distributed_ddl>
        <!-- Path in ZooKeeper to queue with DDL queries -->
        <path>/ch-testch/task_queue/ddl</path>

        <!-- Settings from this profile will be used to execute DDL queries -->
        <!-- <profile>default</profile> -->

        <!-- Controls how much ON CLUSTER queries can be run simultaneously. -->
        <!-- <pool_size>1</pool_size> -->

        <!--
             Cleanup settings (active tasks will not be removed)
        -->

        <!-- Controls task TTL (default 1 week) -->
        <!-- <task_max_lifetime>604800</task_max_lifetime> -->

        <!-- Controls how often cleanup should be performed (in seconds) -->
        <!-- <cleanup_delay_period>60</cleanup_delay_period> -->

        <!-- Controls how many tasks could be in the queue -->
        <!-- <max_tasks_in_queue>1000</max_tasks_in_queue> -->

        <!-- Host name of the current node. If specified, will only compare and not resolve hostnames inside the DDL tasks -->
        <host_name>10.17.0.13</host_name>
    </distributed_ddl>

    <!-- Settings to fine-tune MergeTree tables. See documentation in source code, in MergeTreeSettings.h -->
    <merge_tree>
        <number_of_free_entries_in_pool_to_lower_max_size_of_merge>0</number_of_free_entries_in_pool_to_lower_max_size_of_merge>
        <!-- <max_suspicious_broken_parts>5</max_suspicious_broken_parts> -->
        <!--
        Choose a value between 1024 and 4096.
        The default is 8192.
        -->
        <merge_max_block_size>1024</merge_max_block_size>
        <max_bytes_to_merge_at_max_space_in_pool>1073741824</max_bytes_to_merge_at_max_space_in_pool>
    </merge_tree>

    <!-- Settings to fine-tune ReplicatedMergeTree tables. See documentation in source code, in MergeTreeSettings.h -->
    <!--
    <replicated_merge_tree>
        <max_replicated_fetches_network_bandwidth>1000000000</max_replicated_fetches_network_bandwidth>
    </replicated_merge_tree>
    -->

    <!-- Settings to fine-tune Distributed tables. See documentation in source code, in DistributedSettings.h -->
    <!--
    <distributed>
        <flush_on_detach>false</flush_on_detach>
    </distributed>
    -->

    <!-- Protection from accidental DROP.
         If size of a MergeTree table is greater than max_table_size_to_drop (in bytes) than table could not be dropped with any DROP query.
         If you want do delete one table and don't want to change clickhouse-server config, you could create special file <clickhouse-path>/flags/force_drop_table and make DROP once.
         By default max_table_size_to_drop is 50GB; max_table_size_to_drop=0 allows to DROP any tables.
         The same for max_partition_size_to_drop.
         Uncomment to disable protection.
    -->
    <!-- <max_table_size_to_drop>0</max_table_size_to_drop> -->
    <!-- <max_partition_size_to_drop>0</max_partition_size_to_drop> -->

    <!-- Example of parameters for GraphiteMergeTree table engine -->

    <!-- Directory in <clickhouse-path> containing schema files for various input formats.
         The directory will be created if it doesn't exist.
      -->
    <format_schema_path>/var/lib/clickhouse/format_schemas/</format_schema_path>

    <!-- Directory containing the proto files for the well-known Protobuf types.
      -->
    <google_protos_path>/usr/share/clickhouse/protos/</google_protos_path>

    <!-- Configuration for the query cache -->
    <query_cache>
        <max_size_in_bytes>8388608</max_size_in_bytes>
        <max_entries>1024</max_entries>
        <max_entry_size_in_bytes>1048576</max_entry_size_in_bytes>
        <max_entry_size_in_rows>30000000</max_entry_size_in_rows>
    </query_cache>

    <backups>
        <allowed_path>backups</allowed_path>

        <!-- If the BACKUP command fails and this setting is true then the files
             copied before the failure will be removed automatically.
        -->
        <remove_backup_files_after_failure>true</remove_backup_files_after_failure>
    </backups>

    <!-- This allows to disable exposing addresses in stack traces for security reasons.
         Please be aware that it does not improve security much, but makes debugging much harder.
         The addresses that are small offsets from zero will be displayed nevertheless to show nullptr dereferences.
         Regardless of this configuration, the addresses are visible in the system.stack_trace and system.trace_log tables
         if the user has access to these tables.
         I don't recommend to change this setting.
    <show_addresses_in_stack_traces>false</show_addresses_in_stack_traces>
    -->

</clickhouse>
EOL
      }

      template {
        destination = "secrets/env_vars"
        perms = "644"
        env = true
        data = <<EOL
CH_ADMIN_PASSWORD={{ with secret "epl/data/clickhouse/testch" }}{{ .Data.data.admin_password }}{{ end }}
EOL
      }

      template {
        destination = "secrets/users_config.xml"
        perms = "644"
        data = <<EOL

<clickhouse>
    <!-- Profiles of settings. -->
    <profiles>
        <!-- Default settings. -->
        <default>
        </default>

        <!-- Profile that allows only read queries. -->
        <readonly>
            <readonly>1</readonly>
        </readonly>
    </profiles>

    <!-- Users and ACL. -->
    <users>
        <!-- If user name was not specified, 'default' user is used. -->
        <default>
            <!-- See also the files in users.d directory where the password can be overridden.

                 Password could be specified in plaintext or in SHA256 (in hex format).

                 If you want to specify password in plaintext (not recommended), place it in 'password' element.
                 Example: <password>qwerty</password>.
                 Password could be empty.

                 If you want to specify SHA256, place it in 'password_sha256_hex' element.
                 Example: <password_sha256_hex>65e84be33532fb784c48129675f9eff3a682b27168c0ea744b2cf58ee02337c5</password_sha256_hex>
                 Restrictions of SHA256: impossibility to connect to ClickHouse using MySQL JS client (as of July 2019).

                 If you want to specify double SHA1, place it in 'password_double_sha1_hex' element.
                 Example: <password_double_sha1_hex>e395796d6546b1b65db9d665cd43f0e858dd4303</password_double_sha1_hex>

                 If you want to specify a previously defined LDAP server (see 'ldap_servers' in the main config) for authentication,
                  place its name in 'server' element inside 'ldap' element.
                 Example: <ldap><server>my_ldap_server</server></ldap>

                 If you want to authenticate the user via Kerberos (assuming Kerberos is enabled, see 'kerberos' in the main config),
                  place 'kerberos' element instead of 'password' (and similar) elements.
                 The name part of the canonical principal name of the initiator must match the user name for authentication to succeed.
                 You can also place 'realm' element inside 'kerberos' element to further restrict authentication to only those requests
                  whose initiator's realm matches it.
                 Example: <kerberos />
                 Example: <kerberos><realm>EXAMPLE.COM</realm></kerberos>

                 How to generate decent password:
                 Execute: PASSWORD=$(base64 < /dev/urandom | head -c8); echo "$PASSWORD"; echo -n "$PASSWORD" | sha256sum | tr -d '-'
                 In first line will be password and in second - corresponding SHA256.

                 How to generate double SHA1:
                 Execute: PASSWORD=$(base64 < /dev/urandom | head -c8); echo "$PASSWORD"; echo -n "$PASSWORD" | sha1sum | tr -d '-' | xxd -r -p | sha1sum | tr -d '-'
                 In first line will be password and in second - corresponding double SHA1.
            -->
            <password>{{ with secret "epl/data/clickhouse/testch" }}{{ .Data.data.admin_password }}{{ end }}</password>

            <!-- List of networks with open access.

                 To open access from everywhere, specify:
                    <ip>::/0</ip>

                 To open access only from localhost, specify:
                    <ip>::1</ip>
                    <ip>127.0.0.1</ip>

                 Each element of list has one of the following forms:
                 <ip> IP-address or network mask. Examples: 213.180.204.3 or 10.0.0.1/8 or 10.0.0.1/255.255.255.0
                     2a02:6b8::3 or 2a02:6b8::3/64 or 2a02:6b8::3/ffff:ffff:ffff:ffff::.
                 <host> Hostname. Example: server01.clickhouse.com.
                     To check access, DNS query is performed, and all received addresses compared to peer address.
                 <host_regexp> Regular expression for host names. Example, ^server\d\d-\d\d-\d\.clickhouse\.com$
                     To check access, DNS PTR query is performed for peer address and then regexp is applied.
                     Then, for result of PTR query, another DNS query is performed and all received addresses compared to peer address.
                     Strongly recommended that regexp is ends with $
                 All results of DNS requests are cached till server restart.
            -->
            <networks>
                <!-- eden platform subnet -->
                <ip>10.0.0.0/8</ip>
            </networks>

            <!-- Settings profile for user. -->
            <profile>default</profile>

            <!-- Quota for user. -->
            <quota>default</quota>

            <!-- User can create other users and grant rights to them. -->
            <access_management>1</access_management>

            <!-- User can manipulate named collections. -->
            <named_collection_control>1</named_collection_control>

            <!-- User permissions can be granted here -->
            <!--
            <grants>
                <query>GRANT ALL ON *.*</query>
            </grants>
            -->
        </default>
    </users>

    <!-- Quotas. -->
    <quotas>
        <!-- Name of quota. -->
        <default>
            <!-- Limits for time interval. You could specify many intervals with different limits. -->
            <interval>
                <!-- Length of interval. -->
                <duration>3600</duration>

                <!-- No limits. Just calculate resource usage for time interval. -->
                <queries>0</queries>
                <errors>0</errors>
                <result_rows>0</result_rows>
                <read_rows>0</read_rows>
                <execution_time>0</execution_time>
                <queue_max_wait_ms>1000</queue_max_wait_ms>
                <max_execution_time>10</max_execution_time>
            </interval>
        </default>
    </quotas>
</clickhouse>
EOL
      }

      template {
        destination = "local/init"
        perms = "755"
        data = <<EOL
#!/bin/sh
# helper executable
echo '#!/bin/sh

 clickhouse-client -h 10.17.0.13 --port 8120 --password $CH_ADMIN_PASSWORD
' > /usr/local/bin/connect
chmod +x /usr/local/bin/connect

exec /usr/bin/clickhouse-server --config-file=/secrets/clickhouse_config.xml
EOL
      }
    }
  }

}
