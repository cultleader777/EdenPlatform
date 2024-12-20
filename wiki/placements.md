# Placements

Eden platform allows you to specify on which nodes you want which workloads to run. Unlike other platforms, like kubernetes or raw nomad, eden platform figures out in compile time before deployment if your placements will fail taking into account deterministic memory usage simulation. For instance, if workload requires 10GB RAM and you only have nodes with 8GB RAM, after running simulation placement will fail in compile time instead of deploying and figuring it out in production when your infrastructure is down and you don't know why.

There are three kinds of placements:

## Single node bind

workloads are bound to specific servers. This is inferred for workloads like pg_deployment in which you must expicitly specify which volume of which server the data resides.

Example:
```
 DATA STRUCT pg_deployment [
 {
     deployment_name: testdb,
     WITH pg_deployment_instance [
         {
             instance_id: 1,
             pg_server: server-a=>pgtest1,
         },
         {
             instance_id: 2,
             pg_server: server-b=>pgtest1,
         },
     ]
 }
 ]
```

Here `pg_deploymen_instance` specifies patroni replica and `server-a=>pgtest1` volume will be used to reside this instance data, and, of course placement is `server-a` for instance 1 and `server-b` for instance 2.

## Label placements

if workload is statless, like epl application you can still specify on which specific nodes it will run. Every table that has `placement` or `..._placement` columns can be used to explicitly specify on which nodes you want placement to run. By default these columns allow workload to run on any node.

Say, for `loki_cluster` readers you can specify which labels it will be placed in
```
DATA valid_server_labels {
    server_for_main_loki_cluster;
}

DATA STRUCT loki_cluster {
    cluster_name: main,
    is_region_default: true,
    loki_readers: 1,
    reader_placement: '
      match_keys_and_values:
        server_for_main_loki_cluster: yes_it_is
    ',
    storage_bucket: global=>loki,
    WITH loki_index_scylladb [
       ...
    ]
}
```
Consider `reader_placement` `loki_readers` columns. Reader placement specifies that every server which has server label `server_for_main_loki_cluster` is eligible to run reader workload. Count is the instance count. Note that in eden platform every workload runs on distinct hosts by default.

Also, we must specify in `valid_server_labels` table that such label as `server_for_main_loki_cluster` can exist. This is to avoid cases where you're scratching your head when you made a typo in label on placement and cannot figure out why there's no such server with labels when it is clearly there. You must what labels exist upfront.

To define a server that has label with `server_for_main_loki_cluster: yes_it_is` you can do it like so
```
DATA STRUCT server {
    hostname: server-a,
    ssh_interface: eth0,
    WITH server_label {
        label_name: server_for_main_loki_cluster,
        label_value: yes_it_is,
    }
    WITH server_disk {
        disk_id: '/dev/sda'
    }
    WITH network_interface {
        if_name: eth0,
        if_network: lan,
        if_ip: 10.17.0.10,
        if_prefix: 24,
    }
}
```
child table `server_label` specifies labels that a server has which match loki cluster. Now the stateless workload of loki_cluster will only run inside this server.

## No placement

If your stateless workload has no labels by default it will run on any server that is not marked with run_unassigned_workloads: false
