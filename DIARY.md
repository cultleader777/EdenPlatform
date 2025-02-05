* 2022-08-02

Okay... Representing everything as eden data, valid values and stuff would be tedious...
How about we analyze that data, about valid unit file formats simply from the contents?
Systemd is very concise format of service definition

I can't easily reserve ports in the server and tie them to something else...

Maybe just have it as declaration in the table?

REF count generated column from table? So we could check if only one place refers to this?

Columns for all ref counts and columns for common ref count...

1. implement REF for child elements finding the uniqueness context
2. implement counts for every reference column

WAIT... if we insert into reserved port table, and simply REF to it...
Well, we still need to make sure that there's one thing referring to it...

what is compute order?

Okay, we can check unique constrants before lua checks so we can use them to check reference counts...

We'll need to add extra columns to referred tables?...

Maybe have `_` generated columns for ref counts?..

We can put them after inserting of all data...

Why can't we generate them like parents for rust and stuff?..

We probably need to refactor our serialization vector api.

So we can get unified data to serialize...

* 2022-08-24

DONE: embedded testing database with column store
DONE: ip check that interface belongs to subnet
TODO: force that ip address is not broadcast or network address

* 2022-08-28

TODO: db, every query must work for at least single previous version of db
TODO: db every query must be hitting indexes

* 2022-08-29

DONE: postgres async query don't work, timeout

* 2022-08-30

Fixed stuff, migrations work

TODO: queries

* 2022-08-31

Rock solid pg tests, 2 s setup/teardown

Query arg parsing done, now, analyze!

* 2022-09-01

Query parser with test data prepared
Time to run queries against our DB!

* 2022-09-04

Insertions of data require coercion... Query da thing from DB and try to convert?

* 2022-09-05

Test datasets inserted...

Finally, test queries with test data?

Maybe we can figure out if we're doing seq scans before prod?

DONE: Also, execute multiline statements in migration?

* 2022-09-06

Queries with schemas tested.
How about db mutators and mutating queries?

* 2022-09-07

Mutators up and running.
Querying mutators time? Nothing on api to allow to test...
Materialized views? Just schedule refresh of materialized view every so often.

We can also check if materialized view has refresher!
Check indexes hittage, `EXPLAIN ANALYZE`
We don't cleanup resources well enough yet...
Override docker entrypoint?

* 2022-09-13

DONE: ensure outputs match in tests
DONE: ensure queries are read only
DONE: mutating queries
DONE: ensure mutators don't alter schema

* 2022-09-14

Do I need anything else from db?..
Queries done, mutators done, querying mutators done...
Materialized views?

Let's make sure we have mat views with refresh interval.

* 2022-09-15

Materialized views: done

Typesafe versioned structs?

* 2022-09-19

We we need two channels, that must agree:
1. full struct channel
2. migration channel

snapshot initial:
```
{
    rekt: i64,
    booyah: f64,
}
```

migration:
```
add .hey String
drop .booyah
add .gonzo.boo i64 DEFAULT
rename .rekt .rookt
```

snapshot after:
```
{
    rookt: i64? DEFAULT 123,
    hey: String[],
    gonzo: {
        boo: i64 DEFAULT 321,
    }[]
}
```

* 2022-09-20

DONE: parsing of basic migrations
DONE: parse entire snapshot of type

* 2022-09-21

Parsing structs and migrations.
Check the following:
1. Earliest version of the struct must be a snapshot
2. All fields must be underscored
3. Only valid primitive types are allowed
4. All snapshots but first one must be paired with a migration and must be perfectly matched
5. All fields must be sorted by name when serialized

* 2022-09-26

Done: implemented recursive type system
Refactered stuff, enuf

* 2022-09-27

Okay... Add field done...

DONE: enforce that added field must have default value or be optional.
DONE: default values.
DONE: implement drop/rename fields.

LATER
DONE: We must have time for backward compatibility.
Every struct must have from 0 to N indexes of which field comes when.
Hence, we can still sort but we know which fields come when.
TODO: enums

DONE: fix all the tests, escaping bullshit.

* 2022-10-01

Add indexes to struct fields. We need this now before dropping/renaming fields.

DONE: field index parsing added, fix tests
DONE: test that field sequences are only from 0 to n
DONE: test that no gaps are in field sequences
DONE: test that new added fields are always max plus one
DONE: when removing fields... removing fields doesn't matter?

* 2022-10-03

Okay, the model is clear...
Field order is only relevant when serializing/deserializing.
We should export binary model with code?
Renaming fields only at the same level.

Next up... Applications?

Define application models.
Define queues.
Define databases app uses.
Rest endpoints can have BW structs as fields.

TODO: test parsing of default values to appropriate type?

* 2022-10-10

DONE: application integrated
TODO: start generating project structure

Input structure:
eden db directories (for different environments and stuff)

Project output structure:
/apps/
/bwstructs/
/servers/

Application has:
1. Gen part
2. Impl file

* 2022-10-16

Let's generate bw compat structs, we have cross platform hash
We have ground work laid for generating rust code
Refactorings done

Let's get some snippets!

* 2022-10-17

Struct typegen built.
Serialization function built.
Now functions.
1. switch on first 8 bytes
2. travel pipelines - could be generated at start.

* 2022-10-18

Universion structs work...
TODO: Generate migrations from version to version.

* 2022-10-19

We ensured that migrations can only affect fields once, and that's it.
Any other migration on same field is an error.
Hence, now we can map rename fields one to one.

* 2022-10-20

More or less tested migrations...

Let's keep working on codegen for sample project!

* 2022-10-24

DONE: opt nested fields

* 2022-10-25

TODO: check that http post type name is either:
1. bw compat type
2. empty

* 2022-10-26

DONE: most http parsing
TODO: check if bw compat types exist, and assign these pointers
TODO: generate endpoints?

* 2022-10-27

DONE: cannot start with reserved names /metrics /health
DONE: detect suplicate paths
DONE: compiling http endpoints sketch!
TODO: generate a single struct as all arguments for the input request
TODO: construct unified app api state to pass into request
TODO: booststrap the web server

* 2022-10-29

DONE: json generators

* 2022-10-30

DONE: assert check content type for input body
DONE: parsed json bod
DONE: rendering bodies from functions
DONE: root http app running
DONE: optionally get headers for request

If we need headers from the request we specify they are needed and they are collected and can be used

NEXT: spinup database and nats to see stuff in action
NEXT: generate api methods

* 2022-10-31

DONE: generated db queries/db mutators

* 2022-11-01

DONE: simplified edb, go for nats

* 2022-11-02

TODO: allow edendb default values for primary keys
DONE: create stream if doesn't exist
DONE: generated jetstream producers and consumers
TODO: test the app for real if it works
TODO: db transaction scripts for single database

* 2022-11-03

DONE: prometheus metrics...
We should generate global values for every http, db query and etc.

* 2022-11-04

DONE: Tracking nats message consumption
DONE: Tracking http endpoint times
DONE: Tracking all db queries

DONE: Count http body bytes served
DONE: Count bytes published to nats stream
DONE: Count latency of interacting with nats stream
DONE: Bytes processed when interacting with nats stream

DONE: db transaction scripts

* 2022-11-05

DONE: use connection pool

* 2022-11-07

DONE: sketch of how transaction scripts could work
DONE: mechanism to prevent transaction utilization if sent to other context

* 2022-11-09

DONE: transactions!
Next... test the app?

DONE: TRANSACTION TOTAL TIME BOI

* 2022-11-14

DONE: built one project with nix flakes and crane...
DONE: build single dependency set for apps to be used for rust with nix
cargo2nix seems better choice... we build stuff once?

* 2022-11-15

Petkov stuff seems good, it build slower simply because it was a release build
Have a pet project that is manually built by hand, always use its Cargo.lock file to build stuff
Url arguments should be opt by default and vec if more than one

flake.lock file must be the same across every build!!

Ok, we know how to build... Now, let's build server VMs to deploy?

Generate nixops stuff!

* 2022-11-16

Create base nixos image with password.

Use it as base for the tests.

DONE: generate code to setup the network for test machines
DONE: generate machines shell scripts to raise the infra
TODO: generate scripts to provision machines by the colmena stuff

* 2022-11-17

Okay, we're generating infra.

Few problems:
1. Specify gateway ip for the subnet for VMs. Reserve first ip in the range?
2. Our network names, just assume we will have vlan interface for every server? We should enforce that then.

* 2022-11-18

Two checks:
1. DONE: if subnet is mentioned no other subnet can overlap with it
2. DONE: if subnet is mentioned it must be unique across all subnets
3. DONE: first ip in the subnet is reserved for gateway
4. DONE: root subnets cannot clash... how about internet tho, special case?
5. DONE: check for untruncated networks
6. DONE: internet stuff must have public ips
7. DONE: private stuff must have private ips

Fixed the problems with subnet generation for vms, we calculate first addr in subnet and so on

Okay, we can generate all the needed subnets and define them then in codegen?
Export needed subnets as projection?

Interface -> Subnet

Subnet -> All servers within the same subnet?

* 2022-11-19

DONE: ref child keys for ssh interfaces.
TODO: ref direct child with root disk partitions? Not needed probably...
DONE: build local nix cache... docker container with nginx https://nixos.wiki/wiki/FAQ/Private_Cache_Proxy ?

Plan for deploy:
1. Setup nodes
2. Loop till we can ssh to all of them
3. Spin up nix serve with firewall
4. Run colmena
5. Profit?

* 2022-11-20

DONE: generating everything, dayum!
TODO: deploy nats and patroni stuff?

DONE: do the multiple query args http thingy

* 2022-11-23

TODO: deploy docker cache in host
Almost deployed postgres

* 2022-11-24

DONE: multiple writes/reads in transaction scripts
DONE: consul
TODO: consul tls
TODO: consul acl

We can't make docker container depend on files?
We might have to use nomad...

* 2022-11-27

Okay, we learned how to reload services with systemd units in nix when they're inside keys directory
Entire consul config can be in memory

Okay, so, keyCommand runs command on our local machine to retrieve secrets...
We might need to generate this json from rust and put it to sqlite?..
How about interpolating the thing from secrets db?
We'll need command line to generate template from secrets eventually...

So far let's just use good ol' text

1. Generate consul and nomad configs

* 2022-11-28

Consul works from codegen, nice...
Now, generate and deploy nomad?

* 2022-11-29

Nomad insecure generated.
Now, let's create a secrets engine!

1. Per server?
2. Per infra?

TODO: check that queries hit indexes

* 2022-11-30

Secrets engine created.
Next, generate CA keys/certs and etc?

* 2022-12-01

1. How to generate consul ca secret? We should have a multiple entries in the map?
2. Some secrets can stay string, but others should be deserialized into map?
3. Maybe just support putting certificates and not fetching?..
4. How about adding multiple keys at if one key is not available? There's assert if key exists?

DONE: implemented shell scripting with derivation engine for certs and stuff

* 2022-12-02

DONE: ergonomic multi shell secret derive
DONE: derived all consul tls keys
DONE: consul TLS + ACL tokens!

* 2022-12-03

Nomad certs generated.
Ensure certificates expired, say, after n days and get regenerated?

DONE: nomad tls certs
DONE: dnsmasq to allow consul dns to work
DONE: refactor nix codegen into data, identify components

* 2022-12-04

Refactored stuff out.
DONE: refactor nomad with the new nix plan stuff

* 2022-12-06

DONE: nomad acl tokens

* 2022-12-08

Done nomad ACL bootstrap. Now create policies for namespaces?
Add vault integration?

* 2022-12-10

DONE: nomad anonymous policy and system + epl namespaces bootstrapped

* 2022-12-11

DONE: think of how nomad job deployments should look

1. Have a builder pattern like with nix
2. We must use locked ports in server to issue only unique port addresses
3. We must refer to volumes in server for the job
4. Server port table is not needed then?
5. In templates we can only use reserved ports?

Let's try and see...

TODO: boot on zfs

* 2022-12-12

Scaffold of how stuff would look...

So, we'd have many functions of how to generate nomad job for the postgres, nats and our epl apps

* 2022-12-12

Server volumes... We shouldn't define them by hand, they should be side effects of defining the zfs stuff.
Better yet, materialized view?

I need to rework server volumes to reflect reality...

Just use zfs, make things simpler? If anything else needed we can add?

Root zfs not straightforward...

Make assumptions? Assumption of root volume, nobody cares about that.
But, there can be extra volumes on dedicated zfs disks? Boot non zfs?

1. server_volume is materialized views
2. union of zfs datasets and root volumes
3. we can create arbitrary directories as volumes on root fs
4. each server should have defined root fs capacity

* 2022-12-13

DONE:
1. We can create zfs datasets and expose them to nomad
2. We can expose any root mountpoint to nomad containers with intended contracts

got rid of root fs, just assume it exists, may be zfs or whatever

now we're counting memory, ports, server volumes with the policies, good job!

* 2022-12-15

Almost there...
1. We need to pass secrets to generate consul guids for patroni
2. We need a way to tell to persist certain secret into vault, we can use default KV engine for starters.

Our secrets storage could maybe generate a way to persist certain secret into vault via config?
How to we specify vault tokens tho?
We will inevitably have to deploy vault next.

* 2022-12-16

Most pg stuff generated.
Now we have vault secret requirements, we need to deploy vault and deliver secrets there.
We also need to dump all nomad jobs into separate directory for versioning
DONE: Add postgres synchronous replication config?

* 2022-12-17

Deployed vault, operator init works.

Next: perform operator init automatically and store the outputs in secrets.

* 2022-12-18

DONE: nomad + vault integration
TODO: create kv engine for epl secrets in vault

* 2022-12-19

Secrets engine deployed.

For every component, it needs to reserve it's secret's namespace, perform all it's requests and be done. That's it!

Refactored secrets generation...

Now we can generate single script that:
1. Tests if secret exists
2. If it doesn't exist generates secret and puts it into KV

* 2022-12-20

There must be a vault policy.
If nomad job has secrets it will be according to the job name.

Secret request for apps can work like:
RequestDbAccess {
    database: TableRowPointerDatabase,
    application: TableRowPointerApplication,
}

TODO: proof of deployments doesn't work on our infra?
//PROOF "database is not deployed at all or has minimum two but not more than three replicas" NONE EXIST OF db_deployment_counts {

Base of nomad jobs generated. Do tasks

* 2022-12-21

All off the patroni nomad job seems generated more or less.

TODO: public struct variables may be used to push stuff in inside server runtime...
Restrict or expose fields only as methods.

TODO: if job is stateful it must be batch job
TODO: add haproxy config for patroni
TODO: try running patroni with the file?
TODO: provision needed secrets

* 2022-12-22

DONE: provision script to create secrets
DONE: provision consul policy by secret request
TODO: integrate secret generation to nomad job deployment?

* 2022-12-25

DONE: figure out how to provision consul policy, check if policy exists by name, get source, if different update and create token?
Just check it by the book, does such token exist, does token have policy with such source?

* 2022-12-26

Now consul token provisioning works...
Try to run postgres job? We don't provision zfs volumes yet... Do it with Nix?

* 2022-12-27

TODO: separate table for read only system volumes like certs and etc?
Provisioned volumes for pg.
Let's run it finally?

How to figure out workflow of the vault tokens? Creating new one every time sounds bad...

* 2022-12-28

TODO: generate separate nomad policies for all jobs with their tokens?

Path to renewing vault token:
1. Build new infra
2. Recompile project again for new nix config
3. Run deploy again with new config

Can't access logs without ACL token bois!

Postgres works bois!

DONE: haproxy for master/slave setup

* 2022-12-29

Haproxy done for postgres.
Next, deployments for all nomad jobs?

DONE: docker proxy cache, speed up deployments

* 2023-01-03

DONE: deployment from A-Z, docker containers are running.

Let's make deployments faster with docker pull through cache.

* 2023-01-04

Docker pull through cache done, easy!

Now, generate the application and see if it works?

* 2023-01-11

DONE: nats up and running.
TODO: move two nats errors to platform validation?
TODO: NATS all possible security?
TODO: NATS exporter?

TODO: deploy entire monitoring stack

* 2023-01-26

Minio - up and running.
Create nginx load balancer with least conn stuff?

DONE: nginx load balancer for minio
TODO: minio TLS?
TODO: create bucket and use that for docker artefacts

* 2023-01-27

Added consul service, that is half the battle.

Now we need to manage users for minio.

Probably we need pragmatic approach, if bucket is created it has 3 policites.
1. rw
2. read only + list

So, tables needed:
1. buckets as children
2. policies for each bucket, rw + list
3. users created for each accessor and assigned a policy
4. how to copy secrets to minio deployment, on demand? We could use mc from nix-shell...

DONE: check for no memory assigned to the task

* 2023-01-28

Okay, so we have docker registry table, it has 0 or 1 row.
1. We refer to the claims we want for credentials
2. We MUST propogate vault secret to other keys, hmmmm...

Okay, so we have secrets deployed in order based on dependencies and can copy secret from one to another.

Now, we can deploy docker container and register minio bucket secrets.

Ownership issue, let's deal with this!

Almost there...

TODO: memory analysis for all servers

* 2023-01-29

Almost ready to build, just push to our private registry and try to pull!

TODO: docker registry TLS
TODO: minio TLS
TODO: save images to private repo with alternate tags
Only docker registry image can be OG, it will be cached on every machine anyway.
The rest could be tags?

TODO: Prometheus HA + Victoria Metrics
TODO: add some tests for services?

* 2023-02-02

Building apps.
- Refactor nats queue codegen to allow dashes?

1. Rsync sources after calculating resired hash
2. Run inside the directory `nix build`
3. Load the docker images with `docker load -i result`
4. Retag the image with
```
docker tag hello-world:v0.1.0-wlc0rnr3r900vg302czy9ywk94rvrcr0 epl-docker-registry.service.consul:5000/apps/hello-world:v0.1.0-wlc0rnr3r900vg302czy9ywk94rvrcr0
docker push epl-docker-registry.service.consul:5000/apps/hello-world:v0.1.0-wlc0rnr3r900vg302czy9ywk94rvrcr0
```

Shared rust contexts for apps!!

Bake into the table with children as versions, want more context, just add more versions.

Compute deterministic hash for the build, hash the versions of dependencies, hash root rust version, hash the source inputs.

IDEA: split 64bit key space by table to autoincrement like key starts at 100000000 then at 200000000 for other table and so on,
so we could have key types by returning queries

IDEA: future based row modify api by primary key
```
api.modify_table_x(TableRowPointer, |record_builder| {
   record_builder.set_column_a(1);
   record_builder.set_column_b(2);
})

// returns future of converted tuple values to the final values
api.get_fields_table_x(TableRowPointer, |record_getter| {
   (record_getter.get_column(a), record_getter.get_column(b))
})
```

* 2023-02-05

DONE: validate crate semversions
DONE: validate crate features
DONE: validate nixpkgs version

DONE: push built docker images

TODO: replace tags for apps with generated images in nomad jobs?

TODO: deployments
1. Create DB user
2. Nats user?
3. Deployment table
4. Provision vault secrets for deployments
5. Expose via frontend

* 2023-02-06

DB migrations:

1. Create FWD migration scripts for each schema?
2. Iterate those in order and apply migrations by inserting schema_migrations value with timestamp?
3. SQL can be contained in shellscript?
4. Script:
- Create database if not exists
- Create admin user for the database
- First psql checks if migration exists
- If it doesn't exist it will be performed with script, which is safe from race conditions

BEGIN;

// if migration already run will error out
INSERT INTO schema_migrations(migration_id, started_at, finished_at)
VALUES(123, NOW(), NOW());

// migration body

UPDATE schema_migrations
SET finished_at = NOW()
WHERE migration_id = 123;

COMMIT;

DONE: prevent schema_migrations table from being created

* 2023-02-07

DONE: allow REF columns to have default values in edendb
TODO: remove docker image from postgresql database
DONE: prevent name epl_schema_migrations which is internal table

Migration scripts for each db_schema generated, just apply them to the entire deployment process.

* 2023-02-08

TODO: applications need to know how many shards they can use, there must be unique shard names for each db instance
DONE: move credentials to pg variables

1. Create database if not exists as superuser
2. Create user if not exists as superuser

DB provisioning DONE!!

DONE: ensure queries don't use seq scans in production

* 2023-02-09

All queries now tested for seq scans!

DONE: Check that shard types align with query types
DONE: model is wrong for just checking types.
How about situation when two shards use the same query?
Simply, we should refer to <shard>=><query or mutator or transaction>

Lots of refactoring ahead!

Basically do:
1. application used shard
2. It lists its mutators/queries/transactions

* 2023-02-11

Fully implemented typesafe DB shards!

Now do the same with jetstream queue, application knows only jetstream queue type, that's it!
At deployment we check if types align.

DONE: jetstream streams refactor, disconnect from cluster, make streams typed
DONE: cache jetstream connections if they're the same, assume different NATS connection per user

* 2023-02-12

Nats streams refactorings done!
DONE: nats streams wiring in deployment
Check that streams exist and are of appropriate type

* 2023-02-13

Deployment db wiring done!

Now nats wiring, should be simpler.

* 2023-02-14

Nats stream wiring done!

TODO: generate application deployments?
DONE: move nats stream provisioning to command line instead of rust app (don't have rust max bytes stuff)

* 2023-02-19

DONE: framework for provisioning:
1. nats instances
2. minio instances
3. building apps
4. postgres instances
5. deploying stuff
6. creating vault secrets

Basic framework done. Now factor out the rest of resource provisioning there.

* 2023-02-20

Rewrote all provisionings.
Make sure rsync skips target dir

* 2023-02-21

Provisioning logging fixed!
Provisioning now nats streams!

We can add nats configs or now deploy the application.

TODO: generated consul secrets left in epl provisioning logs
```
[2023-02-21T19:59:34.640610Z] 010 provision-vault-secrets.sh    | Creating acl token with policy epl-pg-testdb
[2023-02-21T19:59:34.747575Z] 010 provision-vault-secrets.sh    | AccessorID:       7f697823-ad72-c99f-ef86-11b2d36c83d8
[2023-02-21T19:59:34.747709Z] 010 provision-vault-secrets.sh    | SecretID:         76cf6be4-3256-4077-806a-6a193f4a9b21
[2023-02-21T19:59:34.747731Z] 010 provision-vault-secrets.sh    | Description:
[2023-02-21T19:59:34.747749Z] 010 provision-vault-secrets.sh    | Local:            false
[2023-02-21T19:59:34.747767Z] 010 provision-vault-secrets.sh    | Create Time:      2023-02-21 19:59:34.721152427 +0000 UTC
[2023-02-21T19:59:34.747787Z] 010 provision-vault-secrets.sh    | Policies:
[2023-02-21T19:59:34.747805Z] 010 provision-vault-secrets.sh    |    5cb57cbf-0186-ddd1-ad4c-586da568bccf - epl-pg-testdb
```

* 2023-02-22

Almost there, generated EPL apps.
Now, when building, how to ensure lockfile exists?
Assert it exists and assert it is checked into source?
When building EPL app progress doesn't stop after error

Two bugs:
1. Postgres credentials password is not propogated to EPL app
2. EPL app cannot connect to NATS jetstream

* 2023-02-23

DONE: enforce that every app has Cargo.lock
DONE: Log errors to user as json
DONE: Handle Postgres NULL as option in return values
TODO: No updates for vault secrets if they changed
TODO: nice UI to connect to all services
TODO: run tests for all apps

* 2023-02-26

Quite a session, did three tasks, Cargo.lock, error logging as json and support NULL as optionals in query outputs

Next: HA Grafana + HA Prometheus + Alert manager

DONE: log fields are unquoted in rust app, hmm

```
{"level":"ERROR","ts":"2023-02-26T20:32:32.421Z","msg":"Error serving http request","error":An internal error occurred. Please try again later.,"endpoint_name":example,"route":/example}
```

DONE: if earlier step fails in provisioning script keeps on going, not good

* 2023-02-28

Think of DNS names!
Okay, so, there are public frontends...
DNS names could be bound to any server?
DNS names could be wildcards too?
We need cnames?

admin.epl-platform.net -> admin panel? with all goodies?
epl-platform.net -> just site?
epl-platform.net/<service name>/ -> exposed services? faster than DNS
epl-platform.net/admin/ -> admin panel, for everyone to reach?

hmm how about just mount services and see if they merge or no?

1. So I need to have TLDs, + wildcards for anything
2. Any server could serve as egress? Or any server could be added the domains needed to meta attribute...
3. Hmmm one service could be root...

Okay, assume two, mount at root and hardcode the admin panel stuff.

TODO: http requests only millisecond resolution, need MOAR!

Okay, route framework sorta good...

Just register static generated admin site with tools we have created?

* 2022-03-02

1. Generate single certificate for root domain *.epl-infra.net
2. All servers can use this certificate for the frontend
3. If there are clashes between server names and subdomains they are detected
4. There can be more TLDs registered, then cert needs to be regenerated?
5. Once we're in production do full legit LetsEncrypt stuff
6. Load balancer uses either LetsEncrypt or it uses self generated certs

DONE: redo server volume locks on runtime.
TODO: check that user volume names don't clash with the system volume names
DONE: expose ca certificates trusted on the server
1. Now it only exposes volumes from DB
2. That should be aggregate of system volumes, like SSL + TableRowPointer volumes
3. How to say I want SSL system volume from the thing? Expose it as an enum probably?

* 2022-03-03

DONE: we have working openresty LB on every server
DONE: generate the admin website
TODO: expose the apps as routes publicly
TODO: make soft reload of nginx configs

* 2022-03-06

Pull docker image before tests or it times out, pg image can't be pulled

DONE: static files under directories in /local/ to paths, no security issues

* 2022-03-07

Okay, idea with // urls flopped completely... sites are loaded incorrectly.
Let's just do bunch of virthosts for admin components!
Except for the root TLD and epl apps, these url schemas we control and can make them work...

I want to expose:
1. TLD + pair
2. It can be sealed or open
3. If it is open we can modify it's paths

Okay, let's expose the EPL apps now!
1. We need separate table to expose these
2. Mount them either under *.subdomain or root tld
3. If empty field of subdomain mount under root
4. Point exposed service to deployments

* 2022-03-13

Exposing stuff:
1. We need to compute the constant route paths as route identifiers
This is the nginx connection, it will be forwarded exactly like that to the upstream app.
We must only check the constant part of the route.
Only dynamic parts can be at the end to make constant routes unique.
For instance
/mookie/1/dookie/4
is illegaly if dookie is a constant route part.
All constants must be upfront.

Hmmm, how about just mount the app but prevent all routes not allowed?
Could be done with openresty...
But nah we generate anyway.

Hmmm, how about we expose everything, all the routes?..
But then there's root route problem anyway...

Calculate route ambiguity for the app?

Say you cannot have routes:
/constanta/constantb
/constanta/argb

Because they're ambigous.
Every route must be identified with unique prefix.
We have when generating code the prefixes, if it's constant or an arg in a path.
We can compute if they're unique.

So we can have binary tree of levels and see if slots clash?
A level in binary tree must have:
1. constant values for separate routes
2. single argument value, any more = error

By this we know for a fact that we can uniquely identify all routes by their constant prefixes!

* 2022-03-14

DONE: http path checker implement merging and all errors associated with that

* 2022-03-15

DONE: http path merging algorithm again with fresh head
DONE: test nested arg checks in merging algorithm
DONE: refactor http merging arg part from vector to option

* 2022-03-16

DONE: rewrite path checking algorithm with current http tree

* 2022-03-21

DONE: write more tests for http path checking
DONE: deal with different HTTP methods in path checking... Have virtual root like this so no clash?

`POST/`
`GET/`

Separate trees for HTTP methods? Doesn't resolve merging issues... Or does it?
Just added separate fields.

TODO: generate nginx configs exposing the endpoints
We need table now of what services we want to expose.

* 2022-03-23

Okay, expose route in tld now?

So, every TLD must have then:
Tree for POST requests?
Tree for GET requests?

Then how do we iterate those trees?..
Merge them by paths all of them?

Okay, first accumulate everything inside the tld subdomain tree:
1. subdomain-a /
2. subdomain-b /

Hmmm... Maybe we should expand our http tree to be method aware?..

We could have two routing principles, one is LOCK everything from that root.
Other one, we could slice and dice and that's it?

Yeah, we should definitely adjust our http tree computations instead...
Then we should be able to merge with ease.

* 2022-03-28

Http path merge engine with different methods done!
Now we can merge the paths in nginx frontend configs for all apps to expose?
Need to redo how we expose admin sites, that they lock root!

* 2022-04-19

TODO: add method that adds page into hierarchy?

We need to split path and check if ends with slash?
If does end with slash, hit the root?
We may also just forward the rest from that path?
Need method to fetch multiple paths of the thing and return what to lock?
Confusing.
There are few things:
1. root page
2. named page
3. forward all to the rest

routes.add_to_path("/", ze_page)

Create iterator for the tree!

* 2022-04-23

Paths dump implemented...

Two problems:
1. Still need to adjust mechanism to allow exposing any page to nginx... Keep the admin pages assert?
2. Multiple nginx methods available inside the location... How to squeeze them all in under same location with a single check?

```
if ( $request_method !~ ^(GET|POST|HEAD)$ ) {
	return 405;
}
```

We can do a aggregation by path in nginx location module, then see all the http methods used and issue one if, the rest we pass unto proxy?
But what if different contents by same method?...
Assert that everything under location has the same content so can use same proxy pass directive!

* 2022-04-24

Almost there, nginx stuff is being exposed now from upstream.

Now, during expose of epl_app_ingress, how to compute the target path?
Think of this...

* 2022-04-25

DONE!! First achy breaky version of app exposure working!

First request:
```
curl -XPOST -H 'Content-Type: application/json' --data '{"some_field":2}' --include -k -H 'Host: www.epl-infra.net' https://10.17.0.10:443/muh/app/example
```
Second request:
```
curl --include -k -H 'Host: www.epl-infra.net' https://10.17.0.10:443/muh/app/hello_world/123/true
```

TODO: Write more test to make sure things work
TODO: Write integration test suite after everything deployed?

What cases won't work now?..

Things could be refactored much more nicely...

TODO: Empty subdomain, what to do with that?
TODO: Hot external LB reload after file change

Doesn't work and can't work on template change? https://discuss.hashicorp.com/t/updating-a-nomad-job-template-for-running-tasks-without-restarting-the-containers/49492

TODO: test with ending slash?

The fact that downstream works is just accident, abstraction is not nice! We assume that target path has argument variables...

TODO NEXT: prometheus + victoria metrics
TODO NEXT: logging with vector

* 2022-04-26

DONE: nginx doesn't start if upstream service is not available

DONE: testing framework carcass

DONE: Integrate testing framework into deployment flow.
DONE: test consul
DONE: test minio
DONE: test postgres
DONE: test nats
DONE: test the demo app
DONE: test epl load balancer

* 2022-04-27

DONE: nginx issue with bad url prefix and variable at proxy pass
DONE: when app can't build deployment is attempted still
DONE: nginx doesn't refresh mirrors with ordinary proxy pass!
DONE: figure out postgresql ports, its a mess

Just integrate the tests into entire workflow and solve nginx issues.

* 2022-05-03

Fixed the issue with url prefix, set variable needs to be above rewrite it seems.
How to incorporate tests into deployment? First we wait for the epl-app-test-hello-world?
Once it is deployed we can run the tests?

Or maybe just ssh and attach until tmux debugging session is done?
Need to have exit status in provisioning?
See if _combined exists for latest dir?

Separated app builds (+ tests) from pushes to docker registry.
Now builds will fail earliest in deployment.

* 2022-05-10

Okay, smaller tasks done, deploy triplets of prometheus + victoria metrics + alert manager

DONE: assert three instances always!
TODO: error on default primary key in edendb compiler?
```
TABLE mon_cluster {
    cluster_name TEXT PRIMARY KEY DEFAULT 'default',
```

DONE: write prometheus integration tests
DONE: expose prometheus service in admin panel
TODO: think of solution for nginx downtime during update.
Possibly just download new nginx config from consul? We could rely on consul...
Consul template?

TODO: Just victoria metrics + alertmanager left

* 2022-05-11

TODO: add healthchecks in nomad to:
- [x] prometheus
- [x] minio
- nats
- vault
- docker registry
- consul
- nomad

TODO: dynamic template reloading in nomad jobs.

Dump entire config into Consul KV and then reload from it dynamically?
Make this abstraction easily available in server runtime?

Now simply do reloads with timed updates for stateful things like postgresql and etc.

TODO: alertmanager secrets for telegram token
TODO: stateful services perform slow rolling upgrades
DONE: implement scraping rules for prometheus, `epl-mon-{cluster_name}` service tag for every cluster scrape

Next up: setup grafana with Postgresql with metric sources and stuff?

* 2022-05-11

TODO: enforce that monitoring cluster is only in single DC
TODO: enforce that monitored nomad jobs are in the same DC always

TODO: add prometheus scraping for
- [x] minio
- [ ] consul - prometheus exporter for consul, but do we need it
- [x] nomad
- [x] vault
- [x] nats - needs deployment
- [x] postgresql exporter
- [x] node exporter
- [x] cadvisor deployment

* 2022-05-12

Added NATS + PostgreSQL prometheus scraping.

Only left node exporter and cadvisor.

TODO: in edendb return added at last expression line, could ruin the expression. Perform more advanced lua expression parsing.

```
        areNumbersInSequence(
            instance_pg_port,
            instance_pg_master_port,
            instance_pg_slave_port,
            instance_patroni_port,
            instance_haproxy_metrics_port,
            instance_postgres_exporter_port
      return ) -- <- RETURN ADDED HERE!!!
```

* 2022-05-20

Added prometheus tests for all metrics of components

* 2022-05-21

DONE: Exposed prometheus metrics for epl apps and for external lb
DONE: Finished with node exporter + cadvisor

* 2022-05-27

Some sort of way to override table defaults is needed. Meta defaults table?
Or just set default as must be defined once by user? Yeah, sounds good...

So far we can hardcode and the more cases the more we get there.

DONE: implement detached defaults

Mark column as DETACHED DEFAULT in the table which must be provided as separate in the source

Every detached default must be defined only once.

Three errors added:
1. Detached default not provided
2. Detached default is wrong type
3. Detached default defined more than once

```
TABLE server {
  hostname TEXT PRIMARY KEY,
  tld REF tld DETACHED DEFAULT,
  something_else INT DETACHED DEFAULT,
}

DEFAULTS {
    server.tld "epl-infra.net",
    server.something_else 777
}
```

DONE: ensure prometheus metric for invalid targets is 0 during testing

DONE: debug why detached defaults don't work

TODO: add datacenter with detached default for DC to verify that monitoring is working inside single region?

* 2022-05-28

TODO: dynamic consul kv reloadable configs abstraction
DONE: any deployment step fails further deployment fails
DONE: integration test that postgresql database exist?
use prometheus exporter?

DONE: unmanaged databases, can be searched in other components
DONE: first grafana deployment

DONE: promxy proxy for all of the metrics from all prometheus clusters
DONE: provision default prometheus dashboards

* 2022-05-29

DONE: prometheus optional metrics path label

* 2022-05-30

DONE: in edendb in lua runtime allow a way to get current source file directory and current source file

For instance, you want to load grafana dashboards next to source file now it doesn't work, edb-src must be hardcoded.

DONE: mechanism for grafana dashboard provisioning

Allow user to add his own custom dashboards with lua glob, problem is we don't know our own directory during lua interpreter run

DONE: integration test showing our dashboard is loaded

Now add more prometheus dashboards, cadvisor is a must

* 2022-05-31

Finished with lua and relative dir stuff.

Now, logging and loki!

Server logs prevent two instances per server, duh!

DONE: scylladb running
DONE: scylladb add passwords
DONE: scylladb register to consul
DONE: scylladb scrape prometheus metrics
DONE: scylladb initialize keyspace for loki
DONE: scylladb add more than 1 core

* 2022-06-01

Stateless memory reservation doesn't match with count? Hmmm... Multiply in analysis?

TODO: Doesn't work when ports are undefined, check silently passes
```
    CHECK {
        local res = areNumbersInSequence(
            loki_writer_http_port,
            loki_writer_grpc_port,
            loki_reader_http_port,
            loki_reader_grpc_port,
            scdb_main_port,
            scdb_main_sa_port,
            scdb_rpc_port,
            scdb_api_port,
            scdb_storage_port,
            scdb_prometheus_port)
        res
    },
```

DONE: loki is somewhat running.
DONE: expose loki consul service
DONE: loki prometheus metrics collection
DONE: add integration tests for loki
DONE: loki second instance non healthy and flaps with timeout on query
restart of instance helped... loki is horrible
removed query frontend, only querier left

* 2022-06-02

Loki seems alright now?
Next up -> add logs from vector to loki

TODO: add loki grafana dashboard
TODO: grafana add pie chart plugin

* 2022-06-03

DONE: add vector journald to loki forwarding
DONE: add docker to loki forwarding
DONE: add epl prov to loki forwarding
DONE: avoid service restarts on key file changes
Could be done by having .src config file and copying to destination with command line during activation?
Done with having separate dir of final keys, ensuring that file path watch systemd unit updates on less than 10 secs old
DONE: integrate loki to grafana

Problems with grafana errors... Tried changing DB, works? Master is switched?..

DONE: loki dashboard not uploaded

Roadmap: High level features of epl
1. Dns
2. Automatic https certs with dns challenge
3. Email
4. x Frontend generation
5. Direct sql expose with sessions
6. Expose S3 minio storage (for email)
7. x Alerting
8. x Logging
9. Theoretical memory distribution html page
10. Docker image table with types
11. Dynamic nginx config reloading
12. Wiki documentation of all capabilities
13. Auto query endpoints
14. User authentications with session

* 2022-06-04

DONE: dumped alerts from prometheus into DB

Sketch tests for alerts:
```
expected_message: 'Le output message for the test'
input_series:
- series: 'up{job="prometheus", instance="localhost:9090"}'
  values: '0 0 0 0 0 0 0 0 0 0 0 0 0 0 0'
- series: 'up{job="node_exporter", instance="localhost:9100"}'
  values: '1+0x6 0 0 0 0 0 0 0 0' # 1 1 1 1 1 1 1 0 0 0 0 0 0 0 0
- series: 'go_goroutines{job="prometheus", instance="localhost:9090"}'
  values: '10+10x2 30+20x5' # 10 20 30 30 50 70 90 110 130
- series: 'go_goroutines{job="node_exporter", instance="localhost:9100"}'
  values: '10+10x7 10+30x4' # 10 20 30 40 50 60 70 80 10 40 70 100 130
```

TODO: edb tricky error
```
TABLE alert_set {
    alertset_name TEXT PRIMARY KEY,
}

TABLE alert {
    alert_name TEXT PRIMARY KEY CHILD of alert_set, // COLUMNS MISSING IF of IS IN SMALL CAPS!!!
    expr TEXT,
    description TEXT,
    for TEXT DEFAULT '5m',
    severity INT DEFAULT 50,
    CHECK { severity >= 1 and severity <= 100 },
}

TABLE alert_trigger_test {
    expected_message TEXT PRIMARY KEY CHILD OF alert,
    input_series TEXT,
}
```

TODO: disallow for column name in eden db because it is reserved rust keyword

DONE: implemented prometheus metrics loading into compile time.
DONE: parse prometheus series for labels
DONE: check prometheus series against existing metrics database
check metric name + labels = profit! Only alerts that can trigger must use only provided inputs

* 2022-06-05

Getting there with prometheus, now I just need:
1. Create single promtool for all tests
2. Run tests and detect failures

TODO: assert eval time in tests
DONE: prometheus exp labels must be exact for test to succeed, bullshit
DONE: integrate prometheus testing into compilation
Basically, now all labels must be matched for test to pass, nonsense
https://github.com/prometheus/prometheus/pull/11033

* 2022-06-06

So, prometheus engine ready. Now integrate alert sets into existing clusters?
DONE: integrated alert groups to existing prometheus clusters
DONE: write test to ensure alerts are monitored

Nextup: FRONT END APPS BOI!!!

* 2022-06-07

Code quality.
1. Memory analsysis so we'd use memory variables
2. Resolve all unused variables issues
3. Hot reload nginx config from consul template

DONE: EdenDB datalog as a turned off feature
DONE: EdenDB generation fix clippy warnings
TODO: EdenDB sqlite warning about forking
TODO: run clippy as part of build

* 2022-06-08

Fix edendb clippy and most of eden platform warnings.
Can finally work on the frontend!

* 2022-06-08

Haven't finished yet with clippy warnings!
DONE: write tests for memory reservation
All server memory reservation is done badly. Just one instance added to stateless memory.

* 2022-06-10

Clippy warnings fixed.

TODO: default stagger for stateful apps
DONE: minimal yew working example with wasm
we can't use tokio but spawn local
TODO: render any hello world yew app and expose it

Okay, almost there with building docker image...
Figure how to serve it with nginx with minimal configuration?

Yew example serving, now separate docker layers so nginx with config is in separate layer.

* 2022-06-11

Skeleton yew/nginx project ready.
Now, make codegen to deploy the hello world, and then I can add codegen for interactions.

DONE: nix environment versions as pointers

promtool missing from shell.nix after upgrade, wat?
Downgrade to previous temporarily...

DONE: added check that alert expression must contain all series defined
No unused series for alerts can be defined now

DONE: ensure nomad job boots up, adjust retry timeout to a minute or so
added buffering for vector, tests should succeed now
DONE: deploy the yew app with nomad
DONE: expose yew app via the path model
TODO: generate stuff to interact with the rest of cors endpoints
TODO: move secrets out of hive.nix to secrets.nix and don't commit that to git

* 2022-06-15

DONE: generate enum for frontend pages
DONE: check that frontend page contains no dupe paths
DONE: expose all frontend pages in nginx (doable now)
TODO: expose same asm page over nginx for different paths

Refactored out http path checking for frontend paths.

DONE: write test for frontend path checking
DONE: write test for duplicate frontend page paths

* 2022-06-16

DONE: grafana doesn't failover to master pg haproxy
add grafana conn_max_lifetime to 300s to make this issue last maximum for 5 minutes
TODO: docker image build is not solid, push doesn't always succeed, these are failed deployments
Added internal frontend tests

DONE: headless chromium test for webassembly app
Next up... Generate page sources and implement page to page navigation in yew.

* 2022-06-17

DONE: yew paths don't support query parameters... Think about how to extract them out?
Now remove...
DONE: forward WASM extension paths and static resources to the frontend app...
Do path level sealing, so that all paths are forwarded to the downstream app?
/*.(js|wasm)
Would need http engine adjustments...
How about regex clashing detection? We know what kind of file it should be named and we can check clashes of any other name to the forwarding rule!

Ok we have now typed pages with arguments.

Next up, forwarding with links from one page to another with typesafe manner?

* 2022-06-18

Fixed the trunk building/serving.

DONE: trunk toml overrride public_url to be relative path (done by manipulating index.html)
https://github.com/thedodd/trunk/blob/7769a17e517391b3b4077c21062725aa1600346d/Trunk.toml#LL11C15-L11C15
Using `public_url = "/"` doesn't work... Empty string either, asset is no served. Why absolute?
```
thread 'tokio-runtime-worker' panicked at 'Paths must start with a `/`'
```

So far I guess we lock?

Trunk build works with relative path, but needs to be overriden for serve

DONE: send compressed wasm/html/js files
we have access to output files now in our build... we can manipulate them any way we want.

DONE: manipulate index.html to have relative paths.
DONE: what to do with page title and input html? Style with template?..
Simply make this as column in frontend_application table

Next up: now we have relative paths, let's make sure our app works from any url.
Create links from one page to another.
Also, our stuff should be wired in somehow of where the urls are from?..
Should index.html be gzipped if it will need to have wiring info from deployment?..

TODO: expose bw compact struct version from every app (to know when migrations can be done and etc.)

* 2022-06-19

Navigation done. Added compression and navigation tests.

Next up: restrict mounting and be able to mount in different places?

* 2022-06-20

DONE: http engine support prefixes

Now use the same to expose ingress route to the frontend app?

* 2022-06-21

TODO: new frontend app is not independently built without cache on the host.

Test cache being down?

DONE: correct way to forward static resources
```
    location ~ ^/other/epl-app-.+$ {
        if ( $request_method !~ ^(GET)$ ) {
            return 405;
        }
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_set_header Host $host;
        set $dummy_var epl-app-frontend-other.service.consul:7438;
        rewrite /other/epl-app-(.+) /epl-app-$1 break;
        proxy_pass http://$dummy_var;
    }
```

Seems like our page doesn't like to be externally routed?

DONE: this url gets relative path https://www.epl-infra.net/other/other/epl-app-41b2eb519564963f_bg.wasm
Not the one under /other/epl-app-41b2eb519564963f_bg.wasm

What to do... Upload wasm artefacts to minio and forward there?
Perform dynamic loading with javascript with relative root?

How about this strategy:
1. Open resty on startup reads the js and WASM file hashes
2. We know our root mountpoint that starts and ends with //
3. Openresty reads index.html to memory... Hmm open resty may be too much.

We can with scripts:
1. Copy the directory
2. Use sed to replace arbitrary string in index.html like OVERRIDE_ROOT_PATH
3. We set the exact mountpoint to OVERRIDE_ROOT_PATH
4. gzip -9 -k index.html
5. Profit?

How about static resources?

Frontend app deployment is not aware of it's path... So can we pass the environment variable to it?
What if mounted on multiple paths? Restrict rule that frontend deployment must have at most one expose? Default to / ?

2022-06-22

1. Enforce that only one egress exists for frontend deployment
2. Specify deployment environment variable target mountpoint

DONE: frontend compile environments are being updated, not good...

Added `--offline` if lockfile exists for `cargo generate-lockfile` command

We need to: add <base href="/relpath/" /> html tag to deployment
Remove . from routes

Okay, now we can mount multiple apps in the paths!

TODO: allow navigation across frontend apps to the different pages
TODO: NEXT perform REST requests to backend services from apps

* 2022-06-24

DONE: Extend sudo credentials in script for automated testing.
DONE: added caching headers

DONE: image url doesn't get replaced for some reason, build failed?

layer already exists and then it doesn't deploy!

TODO: add provisioning script lock in target server

TODO: enable background page load tracking from the user for the frontend

Needs event pipeline!!

DONE: how to handle if service is no longer deployed, we should proxy cache the static website?
We should cache all proxy requests to nginx proxy volume, because frontend app is completely static website.

not needed

TODO: grafana issue keeps reappearing

Current caching policy:
1. If file sizes are different it is recompiled
2. Modification dates are exactly the same always

There can be a subtle bug that file sizes match and etag is the same but we hope that doesn't happen and we have expires 10m directive.

* 2022-06-25

TODO: nix cache in minio for builds

Sketched REST api fetching that would work
Next: specify what we need, implicitly or explicitly?
Maybe types are enough for getting the resources?
Or maybe we should specify REST endpoint?
And search with the same prefix for matching route?
Say /qa/ -> this is app
/qa/api/v1/posts -> this is the API endpoint
Specify the endpoint to imply if its binary or json!!

Okay, almost there with frontend implicit used endpoint searching.
1. We check if under same subdomain and TLD required app exists
2. If more than 1 exists ambigous error is thrown
3. If none exists it is thrown that path doesn't exist

DONE: implement ambigous endpoints wiring to the deployment
TODO: calculate final checked http endpoints for the frontend deployment
TODO: codegen for the wasm calls to the backend

Why are we binding itself to the backend endpoint?

TODO: implement auto backend endpoint!
1. If it is manual it needs to de implemented
2. It can be specified to be auto query
3. It can be specified to be auto mutator
4. It could have context values from authentication or URLs?

* 2022-06-26

Frontend app wirings to backend are figured out.

We just need to generate Rust code now to call those endpoints in a typesafe manner.

* 2022-06-27

DONE: figure out which serialize/deserialize functions are needed
DONE: import eden platform deserialization error
DONE: generate body of the source to call final endpoint
DONE: integrate deployment path adjustments to final frontend apps
DONE: generate function call with closure and spawn callback like in actual implementation

* 2022-07-02

Implement the inside of async function call. Just URL paths are needed and optional body submission.

Gloo net works good, just finish codegen

* 2022-07-03

Mostly working, now add tests ensuring it keeps working from the browser!
DONE: add browser tests that prove REST works from frontend on button press
DONE: run clippy again
TODO: separate server runtime to use enriched intermediate projection?

* 2022-07-04

Implemented frontend with test assurance that contract works...
Now, implement URLs to any target page, if it is html or WASM?

Considerations:
1. URL can point to anything, not just current app.
2. Any domain can be pointed to.

DONE: nginx doesn't fetch next upstream if rollout result is 404 not found
adding this clause didn't seem to help during rollouts, maybe specific entries should be cached in external lb?
```
    if target_path.contains("epl-app-") {
        writeln!(res, "        proxy_next_upstream error timeout http_404 http_500;")
            .expect("Should work");
    }
```

The issue is that javscript is returned 200 OK with our homepage instead!
These should be simply errors for epl-app- stuff and it should work.

* 2022-07-05

Implemented external URL check for `check_frontend_application_page_wirings`.
DONE: write tests for `check_frontend_application_page_wirings` function.
DONE: implement `check_frontend_application_link_wirings`

* 2022-07-07

Written tests for `check_frontend_application_page_wirings`, now need to implement backend link wirings
Implemented function `check_frontend_application_link_wirings` and added tests.

DONE: Codegen with url library in the frontend app
DONE: write working redirection test.
DONE: l1 projections for server runtime refactor
DONE: for link generation post body must be empty
added assert

* 2022-07-10

DONE: use bind instead of dnsmasq

* 2022-07-11

Add DNS names to bind
TODO: global settings table
Can be overriden for tests
How to say you want something else for testing not to include global flags?
DONE: all other server dns over bind
DONE: removed /etc/hosts files based DNS
DONE: generate PTR dns records
TODO: don't generate full zone file for slaves?
TODO: bind architecture for multiple datacenters?
adopt convention that:
10.<datacenter>.<rack>.<place>

We only need to require that servers that reside in DC have the same prefix and then they can have their own dns in-addr zones?
Every DC has its own hostname prefix? Like server-a.dc1.epl-infra.net

* 2022-07-12

DONE: multi dc setup with detached default
DONE: check that all server lan interfaces belong to correct DC subnet
DONE: check all dc cidr clashes and ensure they're /16
TODO: check that all servers have lan interface
DONE: add timer for project deploy and run tests
TODO: edendb singleton tables
1. Only generate row output
2. `SINGLETON TABLE ...` syntax
3. Everything can be the same, just check that one row exists

TODO: multi DC authoritative tld entries from root tld

TODO: on prometheus monitoring cluster we have flag DC default which can be one per DC, boom!
Then users can override it?

TODO: DNS Sec

* 2022-07-13

DNS sec, we're almost there can generate dns sec keys.
We can disable DNSSEC validation for the zones.
Problem is our server has TLD redundantly, but we can choose our datacenter to reside in TLD and forget the thing about having server specify TLD.

TODO: We need to extend database class with traits so we could compute things, like give me TLD of the server.
Or compute the which prometheus instance should be used? Maybe use lazy init as values are the same?

Should work, maybe we can get rid of projections like that also?..

I don't like hiding the state and having no order, now computing projections in order provides predictable compile time errors.

* 2022-07-14

Done big refactorings in DNS

1. Figure out which servers should have DNSSEC keys
- master master server
- masters in slave zones
- masters in every datacenter
- slaves?

TODO: remove zone file definitions from slaves.
Or remove DNS records, just leave SOA stuff

DNS Sec creates many keys automatically, even though they're provided...

Experiment with docker maybe and custom bind config? So that the keys would be found...
Experiment with zone files generated inside nix...

* 2022-07-15

Debugging in docker.
1. Changing permissions to bind:bind made directory writable. Why my keys are not loaded?

-rw------- 1 bind bind  308 Jul 15 06:08 Kdc1.epl-infra.net.+015+2962.key
-rw------- 1 bind bind  179 Jul 15 06:08 Kdc1.epl-infra.net.+015+2962.private
-rw------- 1 bind bind  308 Jul 15 06:08 Kdc1.epl-infra.net.+015+53043.key
-rw------- 1 bind bind  179 Jul 15 06:08 Kdc1.epl-infra.net.+015+53043.private
-rw------- 1 bind bind  301 Jul 15 06:08 Kepl-infra.net.+015+18630.key
-rw------- 1 bind bind  179 Jul 15 06:08 Kepl-infra.net.+015+18630.private
-rw------- 1 bind bind  300 Jul 15 06:08 Kepl-infra.net.+015+36162.key
-rw------- 1 bind bind  179 Jul 15 06:08 Kepl-infra.net.+015+36162.private

Couple of tasks for dnssec:
1. DONE Prepare /run/dnsseckeys directory for bind
2. DONE Copy keys there with exact name
3. DONE Add . to fqdn in ksk/csk files
4. DONE Remove csk from key statement
5. DONE Generate arpa zone ksk/csk keys
6. DONE Zones directory must be writable for bind
7. DONE Add trust anchors static-keys
8. DONE Add dns sec keys to the root zones to validate child zones
9. dnssec integration tests
10. Restart needed for named.service to create directories, not good.

TODO: generate dnscontrol stuff to provision third party services
DONE: dnssec integration tests
DONE: fix the restart needed for named.service so it would resign keys
DONE: slave needs modifiable files for zones

DONE: public internet network interface with DC tests and different DNS views for internet and dc
1. every DC must have at least two servers connected to the internet (redundancy + public DNS)
2. every master DNS server inside a master datacenter must have internet interface

DONE: grafana loki is horrible, inconsistently works in testing when it has all resources needed, try upgrading?
500ms timeout limit in tests, increased to 2 seconds, tests should pass much better.

TODO: when testing we could skip building apps and deploy system components first
we don't do this now because in production before changing any state we want to make sure we have our code compiled.

* 2022-07-16

Added public ips on test cluster.

Have server attributes:
1. egress node, must have public interface
- adjust containers to only run there
- must be at least two per datacenter
2. ensure dns nodes run only on public interface nodes

DONE: test clash of datacenters with ingress hostnames
DONE: move out nix zone configs into separate files, nix zones config don't cut it
DONE: dns auto serial generation
TODO: create internet dns view where we expose external load balancers via zones
TODO: use our own bind dns for testing as oppose to hardcoding OS stuff

* 2022-07-17

We know why every DC needs DNS, but ingress nodes aren't a must in a DC though.
1. DONE If there are ingresses in a DC force at least two nodes per DC to have egress.
2. DONE Deployments must be bound to a DC with detached default?
3. Then we know ingresses for each DC and can add it to the zone files?
4. How do we do a multiregion geo ingress service? Default just goes into the DC it is located at?
5. If ingress is marked multiregion in one dc then it must exist exact same copy across all DCs?
6. DONE Ingress node must have public ip
7. DONE For deployment in datacenter servers must actually exist

Implemented calculation of ingress endpoints from all servers

DONE: ptr dns records for the tld subdomains from ingress
DONE: data prepared, generate public root zone with ingress domains for bind internet view
DONE: integration tests for tld subdomains
DONE: run ingress servers only where is_ingress marker exists on the server

* 2022-07-18

Implemented ingress public dns records. Now filter is_ingress stuff.
Also, after this refactor test project to multiple projects with default env variable.
Then we can add multiple DC setup to test VPN and stuff.

* 2022-07-19

Ingress now only where it needs to be run.
DONE: Refactored test project to allow multiple projects.
NEXT: Multi DC setup with VPN connection connecting subnets via public network?

* 2022-07-20

1. First roadblock, consul server check, ensure that is per DC

Consul federation - ok
Nomad federation - ok
Vault federation, requires enterprise, not needed, will be synced by eden platform.

TODO: make sure consul deployed is aware of its own region and dc
consul doesn't replicate KV across regions
TODO: make sure nomad deployed is aware of its region and dc
TODO: multiple vault secrets will be needed per DC

DONE: after adding region make sure nomad reflects region in certificates
TODO: after adding region make sure consul reflects region in certificates

continue fixing nomad and consul with its secrets

* 2022-07-22

DONE: disallow extra fields in yamls
DONE: rewrite consul checks quorums to have one cluster per region
DONE: rewrite nomad checks quorums to have one cluster per region
DONE: rewrite vault checks quorums to have one cluster per region

consul dns names change with datacenter...
is that an issue?
if we want to reach service in other datacenter but different region, is that bad?
hmm nothing bad should happen I think.
Will postgres break?

okay in consul dc can be just a region and that solves the problems.

DONE: Deployments are now per region, not per datacenter. Make sure our provisioning scripts act appropriately.
DONE: make bind region based fqdn, not dc
DONE: refactor to move tld assignment to region, not dc

* 2022-07-23

DONE: in dns we assumed <second octet>.10.in-addr.arpa zone, one per datacenter, now all datacenter dns must be hosted per region?.. but wait, do we need reverse zones for stuff like that? reverse zone is only for authoritah... nah, do it right. So, we rework dns on the morrow

Let's set the rules straight:
* DNS
- region is single dc mode, at least two per DC
- region in multi dc mode, not more than one per every DC
    DONE: fix dns static analysis
root dns? if one region these are same as DC dns
if multiple regions master and slave is distributed among regions
* Ingresses
- always at least two per datacenter, with public interfaces
* Consul servers
- region is single dc mode, then 3 or 5 in a datacenter
- region is multi dc mode, then 3 or 5 in a region
* Nomad servers
- region is single dc mode, then 3 or 5 in a datacenter
- region is multi dc mode, then 3 or 5 in a region
* Vault servers
- region is single dc mode, then 3 or 5 in a datacenter
- region is multi dc mode, then 3 or 5 in a region

Okay, stuff reworked, refactored, main tests fixed.

Tomorrow:
DONE: rerun and refactor single dc env with integration tests

* 2022-07-24

Almost fixed single dc test!
DONE: Ptr records remaining in dns

Tomorrow: multi dc finally!

* 2022-07-25

DONE: add region and datacenters specification to nomad jobs
TODO: Broke the code. Now propogate regions through all nomad jobs.
TODO: ensure there's default prometheus cluster per region
TODO: ensure there's default logging cluster per region

Docker registry:
DONE: ensure minio deployed is in the same region
DONE: docker registry uses minio from the same region
DONE: pick region for minio cluster and be done with it, then we just check instances

DONE: fix all the broken tests

* 2022-07-26

fixed the tests

DONE: test all errors for monitoring cluster compute
DONE: test all errors for logging cluster compute

DONE: rework vector log forwarding to only have routes to loki clusters inside the region
DONE: pick monitoring cluster for specific nomad jobs

These components need choice for which logging/monitoring cluster they use
DONE: backend apps
DONE: frontend apps
DONE: minio cluster
DONE: nats cluster
DONE: loki cluster
DONE: postgres
DONE: nomad - region default always
DONE: vault - region default always
DONE: consul - region default always
DONE: vector - region default always
DONE: grafana

* 2022-07-27

TODO: rename loki_cluster => logging_cluster
DONE: rename loki_cluster.name => loki_cluster.cluster_name
DONE: minio username is required to be snake case
DONE: maybe reword that frontend_application_deployment_ingress is part of frontend_application_deployment? if we want more frontend apps just have more deployments boi.
DONE: ~we don't scrape nomad consul metrics now~ we do scrape with custom targets
DONE: nats instance quorum must be 3 or 5, no normal error

DONE: tests broken again because of epl regionality stuff...

* 2022-07-29

TODO: first step of deployment, pull all needed images to minio and tag them?
wait, we depend on minio... Minio might be just pulled as is, then bootstrapped...

fixed the bugs

DONE: test that docker registry and minio cant be on diff regions
DONE: ensure app doesn't use database across a region
DONE: ensure app doesn't use queue across a region
DONE: ensure grafana doesn't use database across a region
DONE: ensure loki cluster and bucket are on the same region
DONE: detect multiple usages of minio buckets between logging clusters and docker registry
DONE: ensure loki cluster and loki_index_scylladb are on the same region
DONE: monitoring cluster instances are inside mon cluster region
DONE: ensure nats cluster instances are inside nats cluster region
DONE: ensure db instances are inside db deployment region

DONE: issue vault secret, now needs to be handled per region, refactorings incoming!
DONE: consul policies, now need to be per region!

* 2022-07-30

Most refactored, now generate_machines function!
We'll need to generate provisioning tools and everything per region?

Ton of todos done... How about we go through naming as our schema is relatively stable to see where unifications can be made?

* 2022-07-31

Rename loki_cluster.name to loki_cluster.cluster_name

* 2022-08-01

Sharding rules:
1. Similar purpose columns are same name
2. Every entities primary key must have <something>_name to describe it to know field name in children columns

TODO: stream is persistent, queue is ephemeral

Hmmm, just pick a technology, it is easy to understand then!

backend_application_db_shard.shard_name =>
TODO: rename backend_http_endpoint.http_endpoint_name => backend_http_endpoint.endpoint_name

DONE: db_deployment.deployment_id => db_deployment.instance_id
DONE: scdb_id => instance_id
DONE: scdb_server => scylladb_server
DONE: mon_id => instance_id
DONE: nats_deployment_id => instance_id
DONE: mon_cluster => monitoring_cluster
DONE: remove postgres_databases
DONE: mon_cluster_alert_group => monitoring_cluster_alert_group
DONE: mon_instance => monitoring_instance
DONE: grafana.name => grafana.deployment_name
DONE: datacenter.lan_cidr => datacenter.network_cidr
DONE: network_interface.if_ip <- include cidr subnet mask
kept cidr, shorter name though
TODO: server_volume remove mountpoint, always under /srv/volumes. what about certs?
DONE: versioned_type.vt_name => versioned_type.type_name
DONE: server_zpools.pool_name => server_zpools.zpool_name
DONE: nats_instance_deployment => nats_deployment_instance

Technology to generic renames (probably won't be done)

* 2022-08-02

TODO: fix tests by pulling images if not exist

* 2022-08-03

virt-install breaking change, needs --import flag
cargo lockfile generation nonsense

* 2022-08-06

TODO: move out aux dir to ordinary secrets, with ssh keys and everything
DONE: ssh key always chmod 600
DONE: add default for network_interface.if_ip cidr number to be /24 for DC nodes
TODO: makefile based provisioning for all tasks?
TODO: enforce that you can't create another subnet in datacenter without at least 200 hosts if datacenter is provided by the cloud

* 2022-08-07

Makefiles:
DONE: vm-template
DONE: epl-executable
DONE: docker cache
DONE: nix serve (all interfaces)
DONE: vm networks
DONE: l1 provisioning
DONE: l2 provisioning
DONE: integration tests

Okay running all servers, next ensure we can reach them. Then ensure nix serve and run colmena.

* 2022-08-08

L1 provisioning fully done, l2 left + tests
TODO: fix so far errors encountered during initial prov

* 2022-08-09

TODO: integration tests as an artifact with crane, because recompliation is slow as molassess (not urgent)
DONE: nix-shell doesn't persist inside makefile, that's why we should build integration tests with nix instead.
allow running makefile only inside eden platform shell
DONE: nix-serve stopped working, dafuq?

* 2022-08-10

Almost everything working now, feh, had to use tmux to run nix serve async...
Secondary loki cluster forwarding stopped working, maybe a fluke?
Moved grafana dashboard provisioning at the very end as non essential.

* 2022-08-12

DONE: fixed nasty hidden bug with log forwarding not working due to different loki cluster used because of the same port...
TODO: Try to detect test flakiness?.. Check that if test succeeded first it must always keep succeeding?..
DONE: rename is_vault_instance to is_vault_server
is_vault_instance is good because there are no followers
DONE: split l2 provision script to separate makefile targets
DONE: makefile targets to delete single server
DONE: symlink for makefile and assert assert makefile is used from correct directory

* 2022-08-14

Decided: Makefile and all related operations are in output directory for less confusion
DONE: make mac addresses deterministic per server, we could have issue if we want to simulate removing and adding nodes because now things are sequencial
mac address for interface is generated from sha256(hostname + interface name) (first 48 bits of hash) also set locally administered and unicast bits
DONE: epl tasks name review
DONE: tested server teardown and rebootstrap, works!

* 2022-08-15

DONE: rename is_consul_server -> is_consul_master
DONE: rename is_nomad_server -> is_nomad_master

* 2022-08-16

DONE: consul dns names review, make uniform
DONE: nomad task names review, make uniform
DONE: nomad group names review, make uniform
DONE: nomad job names review, make uniform
only epl-logging renamed to epl-loki
DONE: use dashes everywhere instead of underscores in output directories
convention: directory names in generally dashes, end files can have underscores
DONE: change db_ to pg_ in epl?
db query sounds counter intuitive from the app... keep it pg? we will not add another db than postgres likely.
but wait, clickhouse then enters the picture and foundation db. They're all dbs, user should be aware of which he uses!
hmmm, how about schemas, they could possibly be reused in say sqlite? nah, don't think of cases we don't need yet.

next up: multi dc region! minimum 6 servers, 2 per dc

TODO: first issue in multi dc region:
```
eden platform error: DatacenterWithDeploymentsHasLessThanFourServers {
    datacenter: "dc1",
    servers: [
        "server-a",
        "server-b",
    ],
    minimum: 4,
}
```
This error should be reworked to minimum per region

* 2022-08-17

DONE: makefile target to build all apps
TODO: think what to do with flake.lock, if they diverge... copy from compile environment?
DONE: check to detect duplicate internet ips fail!!
TODO: check that monitoring/logging clusters are spread per region in multi-dc in region mode
TODO: check for unused volumes
TODO: eliminate NetworkAnalysisOutput and move everything into projections

wireguard implementation
DONE: is_vpn_gateway column on server
2 must exist per every datacenter
Every VPN gateway must have public ip interface in the datacenter

DONE: enforce that is_vpn_gateway server has vpn network ip, and enforce its interface name

DONE: generate wireguard nix config file for all peers and their DCs
DONE: remove wireguard interfaces from libvirt generation stuff

* 2022-08-19

TODO: when adding new server it is not added to libvirt network and doesn't work
Now workaround is just to recreate servers, possibly makefile could detect changes
TODO: enable firewall in NixOS boxes
TODO: add ip routes to the gateway VPN servers
ip route add 10.18.0.0/16 via 10.19.0.10 src 10.19.0.12

TODO: find substituters gw ip for every server depending on the network
VM problem, we can't specify single gateway ip for all VMs!.. What if routing works?

Just internet network works, use 77.77.77.1

TODO: first is LAN interface in VMs because all machines have it, then the internet

ping 10.18.0.11 from 10.19.0.12

forward:
10.19.0.12 -> OK -> 10.19.0.10 -> OK -> 10.18.0.10 -> OK -> 10.18.0.11
backward:
10.18.0.11 -> OK -> 10.19.0.11 -> OK


EUREKA:
ip route add 10.19.0.0/16 dev wg0 src 10.18.0.11


FROM 10.18.0.11 and 10.18.0.10 these ping:
ok 10.19.0.10
ok 10.19.0.11
ok 10.19.0.12

FROM 10.19.0.10 only these ping:
ok 10.18.0.10
x  10.18.0.11

FROM 10.19.0.11 only these ping:
x  10.18.0.10
ok 10.18.0.11

FROM 10.19.0.12 only these ping:
ok 10.18.0.10
x  10.18.0.11

good route table 10.18.0.11
```
[root@nixos:~]# ip route
default via 10.18.0.1 dev enp1s0 proto dhcp src 10.18.0.11 metric 1002 
default via 77.77.77.1 dev enp2s0 proto dhcp src 77.77.77.13 metric 1003 
10.17.0.0/16 dev wg0 scope link 
10.18.0.0/24 dev enp1s0 proto dhcp scope link src 10.18.0.11 metric 1002 
10.18.0.0/16 dev wg0 scope link 
10.19.0.0/16 dev wg0 src 10.18.0.11 
77.77.77.0/24 dev enp2s0 proto dhcp scope link src 77.77.77.13 metric 1003 
172.17.0.0/16 dev docker0 proto kernel scope link src 172.17.0.1 linkdown 
172.21.7.0/24 dev wg0 proto kernel scope link src 172.21.7.13 
172.21.7.11 dev wg0 scope link 
172.21.7.13 dev wg0 scope link 
172.21.7.15 dev wg0 scope link 
```

route table 10.18.0.10
```
[root@nixos:~]# ip route
default via 10.18.0.1 dev enp1s0 proto dhcp src 10.18.0.10 metric 1002 
default via 77.77.77.1 dev enp2s0 proto dhcp src 77.77.77.12 metric 1003 
10.17.0.0/16 dev wg0 scope link 
10.18.0.0/24 dev enp1s0 proto dhcp scope link src 10.18.0.10 metric 1002 
10.18.0.0/16 dev wg0 scope link 
10.19.0.0/16 dev wg0 src 10.18.0.10 
77.77.77.0/24 dev enp2s0 proto dhcp scope link src 77.77.77.12 metric 1003 
172.17.0.0/16 dev docker0 proto kernel scope link src 172.17.0.1 linkdown 
172.21.7.0/24 dev wg0 proto kernel scope link src 172.21.7.12 
172.21.7.10 dev wg0 scope link 
172.21.7.12 dev wg0 scope link 
172.21.7.14 dev wg0 scope link
```

route table 10.19.0.10
```
[root@nixos:~]# ip route
default via 10.19.0.1 dev enp1s0 proto dhcp src 10.19.0.10 metric 1002 
default via 77.77.77.1 dev enp2s0 proto dhcp src 77.77.77.14 metric 1003 
10.17.0.0/16 dev wg0 scope link 
10.18.0.0/16 dev wg0 scope link 
10.19.0.0/24 dev enp1s0 proto dhcp scope link src 10.19.0.10 metric 1002 
10.19.0.0/16 dev wg0 scope link 
77.77.77.0/24 dev enp2s0 proto dhcp scope link src 77.77.77.14 metric 1003 
172.17.0.0/16 dev docker0 proto kernel scope link src 172.17.0.1 linkdown 
172.21.7.0/24 dev wg0 proto kernel scope link src 172.21.7.14 
172.21.7.10 dev wg0 scope link 
172.21.7.12 dev wg0 scope link 
172.21.7.14 dev wg0 scope link
```
route table 10.19.0.11
```
[root@nixos:~]# ip route
default via 10.19.0.1 dev enp1s0 proto dhcp src 10.19.0.11 metric 1002 
default via 77.77.77.1 dev enp2s0 proto dhcp src 77.77.77.15 metric 1003 
10.17.0.0/16 dev wg0 scope link 
10.18.0.0/16 dev wg0 scope link 
10.19.0.0/24 dev enp1s0 proto dhcp scope link src 10.19.0.11 metric 1002 
10.19.0.0/16 dev wg0 scope link 
77.77.77.0/24 dev enp2s0 proto dhcp scope link src 77.77.77.15 metric 1003 
172.17.0.0/16 dev docker0 proto kernel scope link src 172.17.0.1 linkdown 
172.21.7.0/24 dev wg0 proto kernel scope link src 172.21.7.15 
172.21.7.11 dev wg0 scope link 
172.21.7.13 dev wg0 scope link 
172.21.7.15 dev wg0 scope link
```


FIXES
```
ip route add 10.18.0.0/16 dev wg0 scope link src 10.19.0.10
ip route add 10.18.0.0/16 dev wg0 scope link src 10.19.0.11
```

ping from 10.19.0.12 to 10.18.0.11 STILL DOESN'T WORK!!
ping from 10.18.0.11 to 10.19.0.12 STILL DOESN'T WORK!!


Objectives for tomorrow:
1. Make sure full mesh ping works between VPN gateways
2. Make sure spoke nodes ping both ways

* 2023-08-20

Works without extra effort
ping -I 10.19.0.11 10.18.0.10

Changes, gw route on 19.0.12
```
 ip route add 10.18.0.0/16 via 10.19.0.10 src 10.19.0.12 scope global
 ip route add 10.19.0.0/16 via 10.18.0.10 src 10.18.0.11 scope global
```


Works without extra effort
```
ping -I 10.18.0.11 10.19.0.12
```

The root of the issue seems that ICMP reply on final server is sent through the wrong interface, should be wg0, cannot yet make it go through better way. There's ip traffic marking also... Traffic for icmp ping is forwarded back to 10.18.0.10 from 10.18.0.11, it should just go through wireguard.

https://serverfault.com/questions/620488/linux-router-ping-responses-going-through-the-wrong-interface
https://forum.openwrt.org/t/wireguard-with-2-wans-responds-through-wrong-wan/119707

When pinging it is received as NAT from gateway, wat? Wireguard doesn't forward?

src 10.19.0.1 dest 10.19.0.12 !!!

going forward through 10.19.0.10 source packet already has wrong ip

Experimental rules with correct gateway
```
 ip route add default via 10.19.0.10 src 10.19.0.12 scope global
 ip rule add to 10.18.0.0/16 table 200
```

```
 ip route add default via 10.18.0.10 src 10.18.0.11 scope global
 ip rule add to 10.19.0.0/16 table 200
```

18.11 -> goes tot eh 18.10 -> ICMP SOURCE IP CHANGED!!! libvirt magicka?


```
ip rule add to 10.18.0.0/16 lookup 80 pref 30
ip route add default dev wg0 table 80
```

```
ip rule add to 10.19.0.0/16 lookup 80 pref 30
ip route add default dev wg0 table 80
```

First hop after gateway ip is screwed!!!!

Wow, libvirt bullshat... Adding these works...
```
sudo iptables -t nat -I LIBVIRT_PRT -s 10.19.0.0/24 -d 10.0.0.0/8 -j ACCEPT
sudo iptables -t nat -I LIBVIRT_PRT -s 10.18.0.0/24 -d 10.0.0.0/8 -j ACCEPT
sudo iptables -t nat -I LIBVIRT_PRT -s 10.19.0.0/24 -d 10.0.0.0/8 -j ACCEPT
```
Try droppin
```
sudo iptables -t filter -I LIBVIRT_FWI -s 10.19.0.0/24 ! -d 10.19.0.0/24 -j DROP
sudo iptables -t filter -I LIBVIRT_FWI -s 10.18.0.0/24 ! -d 10.18.0.0/24 -j DROP
sudo iptables -t filter -I LIBVIRT_FWI -s 10.17.0.0/24 ! -d 10.17.0.0/24 -j DROP
```

Dropping doesn't werk...

Practical solution:
1. In Makefile disable masquerading with iptables rules for all

This one rule works to allow all EPL nodes communicate via wireguard without masquerading stuff
```
iptables -t nat -I LIBVIRT_PRT -s 10.0.0.0/8 -d 10.0.0.0/8 -j RETURN
```

Next steps:
1. add default static route to the first gateway in the DC
2. make consul healthcheck to swap static routes on demand with consul template
3. Install keepalived so we change just one ip instead of many for routing?..
4. Disable automatic routes for wireguard, provision them because we must add source address

* 2023-08-21

MULTIPATH ROUTING BOIS

TODO: add a precondition check to find the network interfaces on server because we rely on correct network interface names on servers to match network_interfaces table. Well, we don't rely yet, activation scripts?

Wait, nexthop load balancing...

Three situations:
1. VPN gateway is inside the subnet, all route traffic there, NO HA!!!
2. VPN gateway is outside the subnet, go to different gateway
3. VPN gateway is in same subnet, load balance between the two

Implement single subnet per DC case now only, most people will never outgrow this.

Okay, all ips ping, but consul isn't bootstrapped yet
TODO: add all servers ping through VPN in makefile?

* 2022-08-22

Consul retry joins fixed per region.
Two checks to be made:
DONE: ensure cross DC pings work
DONE: while consul members doesn't list anything restart consul servers on all machines and wait a few seconds
TODO: write simplified integration tests for Multi DC, that consul works, there are nomad jobs running, that postgres job works that requires consul secrets

TODO: flag inside db deployment to spread over dcs
TODO: flag inside minio deployment to spread over dcs
TODO: flag inside monitoring deployment to spread over dcs
TODO: ensure minio instances use same underlying disks with possibility to disable
DONE: ensure regional ingress spread across dc
DONE: disallow small subnets in DC if more than one subnet (allow override)

Everything werks! Just integration tests are needed...
Next: work out on paper all wireguard configs and routing strategies... Every subnet needs OSPF?..
How about we worry about bare metals and just use OSPF between DCs? Have /23 subnet space for routers and connect all subnets inside a DC...
Need another environment, single region, Multi DC, multi subnet? Two routers at each DC

DONE: lowered VM memory from 8192 -> 4096

* 2022-08-22

OSPF bois!
1. Only inter subnet inside single DC routing, ip range 192.168.12.0/23
TODO: change inter DC VPN ip range to maybe 192.168.21.0/24
2. There must be two routers inside single subnet, VRRP connected
3. There must be two routers inside single subnet, VRRP connected
We need DC router network?..

DONE: enforce that interface has cidr 23 for dcrouter network
DONE: ensure that there are no duplicate ips inside single datacenter for router ip interfaces
DONE: enforce that every server with is_router flag has dcrouter network interface
DONE: enforce that every server with dcrouter network interface is marked with is_router flag
DONE: double server port lock for external LB because it is done per region and is deployed per all regions!!
if comment is the same, assume everything is okay
DONE: test that any server that has vpn interface must have also is_vpn_gateway

* 2022-08-24

DONE: VRRP routing inside da thing
TODO: ensure router VRRP is specified inside router topology with separate table
DONE: ensure dc with only one subnet has no routers
DONE: ensure there are two routers inside every subnet if routing is needed
DONE: parse subnet router floating ips
DONE: ensure router floating ip doesn't clash with nodes inside the subnet
DONE: ensure router floating ip is needed and not unused
DONE: check that server ips cannot be broadcast addresses
DONE: check that server ips cannot be network addresses
DONE: maybe allow OSPF inside single subnet DC?.. this would be for VPN bandwidth scaling, could be overriden with flag
No need, two wireguard servers are enough inside subnet, they can saturate links on bare metals. But wait, how about routing between them?

1. We need HA, VRRP active/passive routers. We can't do double/triple bonding to scale VPN traffic if VPN gateways are also VPN instances.

* 2022-08-26

DONE: enforce if there's at least two DC that we need vpn network
DONE: decide on OSPF areas
OSPF has a single area inside one datacenter which is connected to the backbone area
area 0 - backbone area 192.168.12.0/23
area 1 - dc area like 10.17.0.0/16

DONE: ospf implementation
DONE: validate ospf vtysh command

* 2022-08-27

DONE: add ospf authentication for VPN
DONE: make wg0 links non passive, with heallthchecks, SPF doesn't help us now
DONE: make sure downing wg0 interface works
DONE: if one side of wireguard is downed ospf sees interface is down only in one way but not another
DONE: I probably need VRF to override kernel routes... (no need)

* 2022-08-29

Debugging:

Removing set src rule helped, probably buggy.
```
        !
        !ip prefix-list LAN permit 10.0.0.0/8
        !
        !route-map LANRM permit 10
        !  match ip address prefix-list LAN
        !  set src {lan_iface_ip}
        !
        !ip protocol ospf route-map LAN"#).unwrap();
```
Find way to set source address for routes.

* 2022-08-30

Boom, should werk

```
        !
        ip prefix-list LAN seq 100 permit 10.0.0.0/8 le 32
        !
        !route-map LANRM permit 100
          match ip address prefix-list LAN
          set src {lan_iface_ip}
        !
        ip protocol ospf route-map LANRM
```

DONE: write tests to encapsulate that pings across DCs work now
DONE: add basic tests to ensure that consul/vault/nomad is working
DONE: test intra DC routing with OSPF inside another environment, single DC multiple subnets
DONE: test inter DC routing with OSPF and wireguard inside another environment, single DC multiple subnets
DONE: rewrite OSPF configuration, too much code duplication if node is wireguard gateway or standalone node
DONE: forbid advertising route 192.168.12 to other datacenters
TODO: script to test and teardown all the environments?

* 2022-09-02

DONE: just figure out why 10.19.1.0/24 network is not advertised to 10.19.0.0/24 vpn gateways
Long story short, dcrouter subnet needs to be in the same dc area and then no problem, also just made not to be advertised in config and things work
DONE: Tomorrow add another subnet to dc1, things will 99% just work

* 2022-09-03

DONE: make input directory just part of the output
TODO: refactor to have server types instead of server having its own memory
TODO: fish out unused volumes inside server with okay_unused flag
DONE: finish with passing integration tests for multi dc env
DONE: tests should be generated, not maintained, delete for now

tests now should be minimal because we see that things work and its unrelated to ips

DONE: make libvirt use server memory_bytes attribute

DONE: rename provisioning directory to l2-provisioning in the output
TODO: write wiki networking entry


* 2022-09-04

DONE: wrote first autogen dns test and replaced two tests
DONE: add core count column on the server and reflect that in libvirt

Ran tests in multidc, lag, add two cores?

* 2022-09-05

Almost done with dns tests, now need to workout all subdomains to test against

* 2022-09-05

Finished automating dns tests
Added tcp sockets tests
DONE: add prometheus metric exists tests, refacter lil bit

* 2022-09-07

DONE: add logs exist tests for nats/minio and friens
DONE: nomad/vault/consul/minio/monitoring autogen tests
halfway there, keep going!

* 2022-09-09

  * [ ] Refactored out all generated tests out.

The only tests to add is that stream exists for all jobs in appropriate logging clusters.

* 2022-09-10

fixed logging level
DONE: build integration tests separately in makefile?

* 2022-09-11

AWS:
1. set availability zone for every aws datacenter
2. no aws datacenters can have the same availability zone

DONE: defined aws availability zones
DONE: initial static analysis for aws
TODO: terraform codegen for aws
DONE: all aws availability zones inside region must have the same prefix
DONE: fixed broken tests

* 2022-09-12

DONE: two things, default route to internet gw
DONE: security groups, one public and another one private
TODO: think of what to do with instance types for node
what if we do...
1. table of instance types
2. every aws instance type must be prefixed, say, aws-t2.nano
3. custom user defined instances must have custom- prefix

TODO: add resource names?
TODO: bigger instances and go for full provision
TODO: makefile terraform datacenters
TODO: how to set public ips... track source points in compiler and alter? sed tricks?

* 2022-09-13

TODO: have test vm datacenter implementation, then check that all VMS are either test vm or aws
DONE: db retry on non deterministic error
DONE: setup terraform makefile
TODO: setup terraform vpn

fact: I have to have a way to modify eden data compiler source files.

* 2022-09-16

Adding source replacements done in edendb.
All paths are clear:
- if data is defined from lua just make sure to read terraform state
- if data is defined in edendb sources then it will be replaced with replacements in edendb

TODO: have a step to change the public ips after terraform run once we know the public ips

* 2022-09-17

Fix vm image generation, check if path exists?

terraform state exists, we need program to generate replacements and modify

One jq query to crete aws replacements:

```
cat terraform.tfstate| jq '
{"network_interface":[ .resources[] | select(.type | contains("aws_instance")) | select(.instances[0].attributes.public_ip != "") | {"primary_key": (.name + "=>eth1"),"replacements":{"if_ip":.instances[0].attributes.public_ip}} ]}'
```

Sample output
```
{
  "network_interfaces": [
    {
      "primary_key": "server-c=>eth1",
      "replacements": {
        "ip_address": "123.123.123.123"
      }
    },
    {
      "primary_key": "server-d=>eth1",
      "replacements": {
        "ip_address": "123.123.123.124"
      }
    }
  ]
}
```

DONE: problem in EPL, network interfaces are children of the server, I must implement replacements by child primary keys

DONE: in makefile add replacements processing script for aws to generate replacements json
DONE: call edendb compiler to perform public ip terraform replacements

* 2022-09-18

DONE: in aws enforce that every single public ip interface is named the same for replacements, aws doesn't give us interface inside vm anyway
DONE: deployment.targetHost set public ip if available
DONE: enforce in aws that ssh interface for public nodes is public ip?
DONE: think of what to do with ssh config when provisioning aws
DONE: only 3 gigs of disk inside aws machine, add like 20G for starters?
DONE: copying derivations to remotes is slow as molassess
TODO: setup sshuttle and then push to the rest of the hosts
TODO: setup nix cache server on remote?

* 2022-09-19

DONE: expose private instances via nat gateway to the internet
TODO: reserve first 4 ips from instances in aws config

* 2022-09-21

This ip tables rule works for natting
```
table ip nat {
        chain postrouting {
                type nat hook postrouting priority 0;
                ip saddr 10.0.0.0/8
                ip daddr != { 10.0.0.0/8 }
                masquerade;
        }
}
```

```
nft -f rools.txt
```

DONE: in aws case we must have subnet routers
DONE: disable dhcp in aws, assign ips manually to interfaces
TODO: ip table reassociassion from the nodes as per haproxy article
https://www.haproxy.com/blog/haproxy-on-aws-best-practices-part-3#:~:text=VRRP%20is%20a%20protocol%20for,a%20multicast%20overlay%20with%20n2n.


assign private ip
```
aws ec2 assign-private-ip-addresses --allow-reassignment --network-interface-id eni-xxxxx --region us-west-1 --private-ip-addresses 10.17.0.7
```

Da script
```
#!/bin/sh

VIP=10.17.0.7
IF=enX0

MAC=`ip link show $IF | awk '/ether/ {print $2}'`
ENI_ID=`curl http://169.254.169.254/latest/meta-data/network/interfaces/macs/$MAC/interface-id`
export AWS_DEFAULT_REGION=`curl http://169.254.169.254/latest/meta-data/placement/availability-zone | rev | cut -c2- | rev`

echo $MAC
echo $ENI_ID
echo $AWS_DEFAULT_REGION

aws ec2 assign-private-ip-addresses \
        --allow-reassignment \
        --network-interface-id $ENI_ID \
        --private-ip-addresses $VIP

```

DONE: edendb non valid snake case fkeys push to master
TODO: ensure aws reserved ips don't clash with ours
TODO: add aws dc routes to local aws default gateway
TODO: what to do so aws internal ips are not accessed by containers?

The provisioning plan:
1. provision public servers
2. open sshuttle to subnets
3. background job for an hour to keep setting default gateway to our custom servers
4. do l1 provisioning on all servers via sshuttle

* 2023-09-23

DONE: router network bridges for 192.168.12.0/23 network don't work!!!

Example attach iface
```
aws ec2 attach-network-interface \
    --region=us-west-1 \
    --device-index=1 \
    --instance-id=i-0000 \
    --network-interface-id=eni-0000
```

1. Check if already attached to our instance


```
# get the attachment id
aws ec2 describe-network-interfaces --network-interface-ids=eni-0000 --region=us-west-1 | jq -r '.NetworkInterfaces[0].Attachment.AttachmentId'
```

DONE: detaching/reattaching interface is slow (~20 seconds), are the better options to route private subnets?
but we don't use public traffic that much in EPL, its only when nodes want to access the internet.

1. we have interface,

DONE: temp routing solution with ssh
```
while true
    ip route del default via 10.17.0.1 dev enX0 proto dhcp src 10.17.0.11 metric 1002 mtu 9001
    ip route add default via 10.17.0.7 dev enX0 proto dhcp src 10.17.0.11 metric 1002 mtu 9001
    sleep 1
done
```

* 2023-09-30

TODO: generated tests specify explicit types if targets are empty, make sure tests without targets don't exist at all?

DONE: colmena is way too slow to be acceptable, manual copy secrets, nixos-rebuild switch is better
1. rsync secrets to remote dir
2. rsync nixos file to /etc/nixos then perform rebuild switch with fast internet

generate_hive_nix receives secrets as input!!

Deleted testing keys, fix all compile time errors, generate keys

* 2023-10-07

Refactored out test public keys

* 2023-10-08

DONE: how to preserve file for secrets...
1. execute provisioning, it might fail
2. mkdir -p -m 0700 /run/keys
3. rsync the script with restrictive root permissions /run/keys directory
4. run another ssh script

either way, just one ssh blob!!! let's do compressed ssh script we ungzip

So, one ssh blob:
1. Generate plaintext ssh in memory for every server
2. gzip -9 the plan before transfer
2.1. 'echo <gzipped ssh plan> | ssh root@<server ip> 'nix-shell -p gzip j'
`echo 'echo mookie' | gzip -9 | nix-shell -p gzip --command 'gunzip | sh'`

`echo '<uncompressed input>' | gzip -9 | nix-shell -p gzip --command 'gunzip | sh'`

* 2023-10-14

`echo '<uncompressed input>' | gzip -9 | nix-shell -p gzip -p sqlite -p tmux --command 'gunzip | sh'`

Alright, l1 async provisioning done, now add makefile entry to await for l1 provisioning entry to be finished and we're good

Resolve next error:
```
[root@nixos:/var/lib/epl-l1-prov]# cat xxx.log
/etc/nixos ~
hint: Using 'master' as the name for the initial branch. This default branch name
hint: is subject to change. To configure the initial branch name to use in all
hint: of your new repositories, which will suppress this warning, call:
hint:
hint:   git config --global init.defaultBranch <name>
hint:
hint: Names commonly chosen instead of 'master' are 'main', 'trunk' and
hint: 'development'. The just-created branch can be renamed via this command:
hint:
hint:   git branch -m <name>
Initialized empty Git repository in /etc/nixos/.git/
[master (root-commit) 920502c] Update
 1 file changed, 479 insertions(+)
 create mode 100644 configuration.nix
~
building Nix...
building the system configuration...
error:
       Failed assertions:
       - You must set the option ‘boot.loader.grub.devices’ or 'boot.loader.grub.mirroredBoots' to make the system bootable.
(use '--show-trace' to show detailed location information)

```

* 2023-10-15

L1 provisioning works
DONE: l1 provisioning sqlite lock and exit on error
DONE: redirect l1 provisioning logs to vector
DONE: l1 provisioning log rotation
DONE: remove key syncer service, we manually control secrets copying
TODO: l2 logs delete after sending to vector
TODO: how to cache artefacts without colmena
TODO: correct locking on checking if l1 provisioning already exists?
DONE: clean tmp secrets

* 2023-10-22

Almost there with AWS, fix docker on nodes with public ips. done!
Minio doesn't work. Works!

Everything seems to work, sshuttle vpn sucks. How about we do wireguard to remote servers instead?

DONE: use wireguard instead of sshuttle for better remote connections
TODO: think of how to distribute provisioning through nats queues for every server, encrypt with every servers gpg key and generate keys for every server?

* 2023-11-12

wireguard vpn we can setup different admin keys for VPN then connect from any node, what if ip clashes? we need range for vpn network?

DONE: reserve one vpn ip at the end of the range
DONE: generate client wireguard config with all the DCs
DONE: makefile to up vpn and down vpn
DONE: wg config remove [Interface] section
DONE: changed the wireguard to provision even if only one dc for vpn

Wireguard setup needs to be done by hand by this guide, we can load peers from config, but private keys only from command line https://www.wireguard.com/quickstart/

* 2023-11-17

Setup nailed down for local wg, do this in Makefile
```
#!/bin/sh

ip link add dev wg7 type wireguard
ip address add dev wg7 172.21.7.254/24
wg set wg7 private-key /tank/projects/p1/test-envs/envs/aws-single-dc/admin-wg-private-key
wg setconf wg7 admin-wg.conf
ip link set up dev wg7
```

We can't connect to the remote servers though, telnet doesn't work, telnet for ssh should succeed on internal ips

* 2023-11-19

Issues:
1. Wireguard systemd unit not loaded, peers missing
2. After peers are established only VPN endpoint itself works, we need masquerading
3. Maybe just have one systemd unit that executes our setup because now it is failing?

In wireguard client we just need up rule, that's it

No need of this, already have NAT routing!!
```
        chain EPL_WIREGUARD_ADMIN {
                type nat hook postrouting priority srcnat; policy accept;
                ip saddr 172.21.7.254 ip daddr 10.17.0.0/16 iifname "wg0" masquerade
        }
```

Copy paste edit this script for systemd service and we're good
```
set -e
modprobe wireguard || true



ip link add dev "wg0" type wireguard



# this fails, add || true ?
ip address add "172.21.7.10/24" dev "wg0"

wg set "wg0" private-key "/run/keys/epl-wireguard-key" listen-port "51820"

ip link set up dev "wg0"
```

Wireguard works now!

DONE: allowed ICMP pings for aws public nodes

DONE: left dns errors are mystical, DNS responds on site correctly but not when locally from laptop with VPN, rebuild env, it was thingy with current VPN node forwarding from VPN ip

Hmm what to do with secrets that can't be generated before provisioning even though there are no dependencies... if we could know uid/gid in advance that is solved

DONE: configure nomad frontend by the book, disable proxy_buffering https://developer.hashicorp.com/nomad/tutorials/manage-clusters/reverse-proxy-ui

DONE: test aws transit gateway

* 2023-11-20

Started experimenting with transit gateway.

Everything works but thanks to ospf and wireguard.

DONE: We need to use transit gateway though.
DONE: We have wireguard leftovers with l1 provisioning
move extra past wait

* 2023-11-21

Ran the transit gateway tests, werks
TODO: add basic auth for admin panel
TODO: azure doesn't accept prepaid cards (faggots), get a credit card, in the meantime google cloud

* 2023-11-26

DONE: google cloud static analysis + tests
DONE: generate test terraform infra and modify codegen for google cloud
DONE: add project id inside google stuff, we need global settings table
DONE: build NixOS image for google cloud
DONE: upload NixOS image to google cloud

1. build image locally
2. upload as part of makefile flow
3. hardcode in terraform

Mainainer script mentioned in https://nixos.wiki/wiki/Install_NixOS_on_GCE
```
#!/usr/bin/env nix-shell
#! nix-shell -i bash -p google-cloud-sdk

set -euo pipefail

BUCKET_NAME="${BUCKET_NAME:-nixos-cloud-images}"
TIMESTAMP="$(date +%Y%m%d%H%M)"
export TIMESTAMP

nix-build '<nixpkgs/nixos/lib/eval-config.nix>' \
   -A config.system.build.googleComputeImage \
   --arg modules "[ <nixpkgs/nixos/modules/virtualisation/google-compute-image.nix> ]" \
   --argstr system x86_64-linux \
   -o gce \
   -j 10

img_path=$(echo gce/*.tar.gz)
img_name=${IMAGE_NAME:-$(basename "$img_path")}
img_id=$(echo "$img_name" | sed 's|.raw.tar.gz$||;s|\.|-|g;s|_|-|g')
img_family=$(echo "$img_id" | cut -d - -f1-4)

if ! gsutil ls "gs://${BUCKET_NAME}/$img_name"; then
  gsutil cp "$img_path" "gs://${BUCKET_NAME}/$img_name"
  gsutil acl ch -u AllUsers:R "gs://${BUCKET_NAME}/$img_name"

  gcloud compute images create \
    "$img_id" \
    --source-uri "gs://${BUCKET_NAME}/$img_name" \
    --family="$img_family"

  gcloud compute images add-iam-policy-binding \
    "$img_id" \
    --member='allAuthenticatedUsers' \
    --role='roles/compute.imageUser'
fi
```

* 2023-12-03

DONE: utility script to clone one environment to the next
DONE: storage bucket location, should be maybe the first dc?
DONE: google cloud storage not configured
TODO: add google cloud check for terraform credentials

* 2023-12-04

DONE: google project id must be exact in terraform
DONE: build gce image with embedded ssh key and disable gcloud ssh key management

```
nix-build '<nixpkgs/nixos/lib/eval-config.nix>' \
   -A config.system.build.googleComputeImage \
   --arg modules "[ <nixpkgs/nixos/modules/virtualisation/google-compute-image.nix> ]" \
   --arg config "{ inherit (config.virtualization.googleComputeImage); configFile = \"./mcpezzlow2.nix\"; }"  \
   --argstr system x86_64-linux \
   -o gce \
   -j 10
```

* 2023-12-09

TODO: check if bucket picked for google for image already exists, perform a check in preconditions stage
DONE: decide upon gce image location
DONE: unified global settings tuple

DONE: extract terraform private ips for gcloud
```
cat terraform.tfstate.backup|jq '{"network_interface":[ .resources[] | select(.type | contains("google_compute_instance")) | select(.instances[0].attributes.network_interface[0].access_config[0].nat_ip != "") | { "primary_key": (.name + "=>eth1"), "replacements":{"if_ip":.instances[0].attributes.network_interface[0].access_config[0].nat_ip} } ]}'
```

TODO: l1-provision-public-nodes make provisioning in parallel

DONE: next archival error during l1-provisiong, out of disk space

```
error: failed to extract archive (Write failed)
(use '--show-trace' to show detailed location information)
```

DONE: gcloud single dc tests passing
DONE: gcloud multi dc tests
DONE: aws + aws + gcloud tests
DONE: how VRRP works in gcloud?

google_compute_route to route packets for outside DCs to gateway instances which have VRRP
https://geko.cloud/en/setup-haproxy-googlecloud-keepalived/

* 2023-12-10

TODO: matrix of RAM/CPU settings for cloud instances and enforce checks on them
CANCELLED: allow VRRP alias ip range for compute instances
CANCELLED: create terraform routes for all other clouds but gcloud
Okay, cloud failover is slow as molassess.
How about, VRRP router servers publish new routing rules directly to consul for routing table?

Router publishes rules to the consul
Consul watch works bois!

* 2023-12-16

Consul based node inter DC routing

DONE: vrrp machines publish routes to themselves on success at consul key
One consul per region!
/epl-interdc-routes/<datacenter>
DONE: create consul ACL for DNS route keys, readable by anyone, writable by DC vrrp
DONE: create consul ACL token to write to those keys per every datacenter
DONE: adjust VRRP script to publish the routes using the token on specific service
DONE: create consul kv watch daemons to delete old routes and replace with new
We should have state of previous routes to remove

Develop consul stuf with this notification
```
{"Key":"epl-interdc-routes/dc1","CreateIndex":349,"ModifyIndex":4332,"LockIndex":0,"Flags":0,"Value":"CiMgUk9VVEVTIENSRUFURQppcCByb3V0ZSBhZGQgMTAuMTguMC4wLzE2IHZpYSAxMC4xNy4wLjEwCmlwIHJvdXRlIGFkZCAxMC4xOS4wLjAvMTYgdmlhIDEwLjE3LjAuMTAKCiMgUk9VVEVTIERFTEVURQppcCByb3V0ZSBkZWwgMTAuMTguMC4wLzE2CmlwIHJvdXRlIGRlbCAxMC4xOS4wLjAvMTYKCiMgRklOSVNICgo=","Session":""}
```

DONE: have a matrix and workout when do we need epl consul routing
Usecases:
1. Redundant internet gateway in AWS
2. To reach remote datacetners

DONE: run google cloud + google cloud + aws setup
DONE: AWS lan interace must be named enX0, we must have precondition checks
DONE: make keepalived to wait for consul availability
DONE: custom keepalived daemon with custom config, doesn't react to file changes?
DONE: big timeout while rebuilding configs on private nodes, activation scripts?
DONE: remove need for floating ips in clouds where consul VRRP is needed?
TODO: eden platform error: ServerIpCannotBeBroadcastAddress for public ip, make sure terraform updates subnet mask to /32 for cloud internet nodes
TODO: make sure NixOS activation doesn't wait for ready services
DONE: set initial routes of can't reach consul k/v via activation script
DONE: internal servers g/h/i lose connection after provisioning... can ping but can't ssh
DONE: something was wrong with the activation script for bootstrapping routes, figure it out, we will need it in the future, because we can't bootstrap consul for satelite DCs without inter DC connection. Remove umask?

DONE: after consul bootstrap publish routes to KV store right away, don't wait for failover

* 2023-12-17

Internet gateway = Wireguard node

Three models of datacenter:
- Multiple subnets
To reach internet for nodes:
1. forward traffic to subnet router, it will know which route leads to internet
2. cloud native NAT
To reach other DCs for nodes:
1. If inside the same subnet as the NAT forward to VPN gateway
2. If outside the subnet of the NAT forward to the subnet router, it will know where is the wg gateway

- Single subnet
To reach internet for nodes:
1. forward traffic to wireguard gateways
2. cloud native NAT
To reach other DCs for nodes:
1. forward traffic to wireguard gateways

How many routes we need:
1. Route to reach other DCs
2. Route to the internet

If we're on different subnets than the VPN we could introduce internet gateway type
We could always have internet gateway, and if enforce that wireguard gateway is also VPN gateway!
So, if internet gateways exists on the subnet all nodes route to that, we compute routes for every subnet

DONE: run tests with multiple AWS subnets inside DC and make sure everything works, + develop internet gateway for subnets?

DONE: next hop issue in google
```
[root@server-g:/var/log/epl-l1-prov]# ip route add 10.19.0.0/16 via 10.17.0.10 src 10.17.0.12
Error: Nexthop has invalid gateway.
```

* 2023-12-19

Internet from GCP to AWS is horrible!!!
```
[nix-shell:~]# iperf -c 172.21.7.14
Connecting to host 172.21.7.14, port 5201
[  6] local 172.21.7.10 port 35536 connected to 172.21.7.14 port 5201
[ ID] Interval           Transfer     Bitrate         Retr  Cwnd
[  6]   0.00-1.00   sec   203 KBytes  1.66 Mbits/sec    0   20.0 KBytes
[  6]   1.00-2.00   sec   164 KBytes  1.35 Mbits/sec    0   20.0 KBytes
[  6]   2.00-3.00   sec   231 KBytes  1.89 Mbits/sec    0   30.7 KBytes
[  6]   3.00-4.00   sec   163 KBytes  1.34 Mbits/sec    0   30.7 KBytes
[  6]   4.00-5.00   sec   182 KBytes  1.49 Mbits/sec    0   30.7 KBytes
[  6]   5.00-6.00   sec   136 KBytes  1.12 Mbits/sec    0   30.7 KBytes
[  6]   6.00-7.00   sec   182 KBytes  1.49 Mbits/sec    0   30.7 KBytes
[  6]   7.00-8.00   sec   323 KBytes  2.65 Mbits/sec    0   42.8 KBytes
[  6]   8.00-9.00   sec   273 KBytes  2.23 Mbits/sec    0   42.8 KBytes
[  6]   9.00-10.00  sec   182 KBytes  1.49 Mbits/sec    0   42.8 KBytes
- - - - - - - - - - - - - - - - - - - - - - - - -
[ ID] Interval           Transfer     Bitrate         Retr
[  6]   0.00-10.00  sec  1.99 MBytes  1.67 Mbits/sec    0             sender
[  6]   0.00-10.04  sec  1.86 MBytes  1.56 Mbits/sec                  receiver
```

DONE: ens5 interface for m5zn.large!!
TODO: when building gce image make sure it uses NixOS version from data

* 2023-12-20 networking model

1. Every subnet has two routers, they may have exits to the internet
2. Every subnet node has two questions:
- Do I need internet from EPL routers? Is internet managed by cloud?
- Do I need routes to the outer outside DCs?
3. Routers are in separate subnet, let's use existing like 10.17.254.0/23 which is hidden from other routers
this way VPC has no excuse to not allow us to use it?

TODO: aws <-> gcloud speeds are horrible, we need to be able to scale bonds to the networks
Maybe consul should list routes available and do equal cost lb?

DONE: build gcloud image with custom version
DONE: build aws image just like google cloud
DONE: upload and use our custom aws image
DONE: aws artefact bucket move to project setings
TODO: aws image is 1.2GB, with compression only 400MB, maybe possible to upload less to S3?

Oy vey, image import in aws is madness!

➜  aws git:(cloud_gen@gcloud@vrrp-consul) ✗ aws ec2 describe-import-image-tasks --region=us-west-2 | jq
{
  "ImportImageTasks": [
    {
      "Description": "testimage",
      "ImportTaskId": "import-ami-xxx",
      "SnapshotDetails": [
        {
          "DeviceName": "/dev/sde",
          "DiskImageSize": 1468371456,
          "Format": "VHD",
          "Status": "completed",
          "UserBucket": {
            "S3Bucket": "...",
            "S3Key": "nixos-amazon-image-23.05.20230531.4ecab32-x86_64-linux.vhd"
          }
        }
      ],
      "Status": "deleted",
      "StatusMessage": "ClientError: Unable to find an etc directory with fstab.",
      "Tags": []
    }
  ]
}


huge script to upload images... just reuse it
./nixos/maintainers/scripts/ec2/create-amis.sh

DONE: provision the unique named vmimages role for aws to avoid project clashes

aws import fails!!!
```
# home_region=us-west-2 bucket=dask-1329-boo service_role_name=vmimport ../../../misc/create-amis.sh terraform/aws/result
```

in aws we need to import image per region!!

DONE: disable checking for image availability, modify the script

An error occurred (OperationNotPermitted) when calling the ModifyImageAttribute operation: You can’t publicly share this image because block public access for AMIs is enabled for this account. To publicly share the image, you must call the DisableImageBlockPublicAccess API.

DONE: aws image from cache if exists for multiple runs
DONE: remove aws ssh key management stuff, key wired to image
DONE: if ami already exists don't upload the image, just upload small file
DONE: since be build our own VMs, add gzip/sqlite and tmux into images for fast provisionings
DONE: aws/gcloud internet interface names should be specified as 'void' to avoid conflict with router interface names

* 2023-12-21

DONE: workout networking model with simulation tests
TODO: connect vpn gateways by sorting LAN ips

* 2023-12-23

TODO: rename is_subnet_router -> is_subnet_gateway

because in AWS it is not a router, but it is responsible only for subnet internet

Tests are now fixed. Next, delete old routing algorithm and refactor

DONE: add route to the internet for all subnet gateways that have internet ip

* 2023-12-24

TODO: for test VMs make sure to instantiate server configurations to make sure all cache things are available

We can generate configs like
```
nixos-generate -c ./configuration.nix --format vm
```

DONE: test case for multi dc OSPF routing config
DONE: add interface preconditions

If we moved routing subnet to predefined ranges we avoid qemu problem of not being able to use DHCP to create multiple same networks on same machine!

Not firewall rule... Figure out why hosts don't ping


TODO: enable soft mem limits for nomad cluster
DONE: keepalived still not working first time
DONE: optimize the case where all traffic to 10.0.0.0/8 can go to the one gateway

* 2023-12-26
Provisioning id 20231226170636 not found in l1 provisioning database
Parse error near line 1: database is locked (5)

DONE: regions and testing simulated configs
DONE: unified firewall config from NixOS side

TODO: nomad/consul cross region federation
DONE: nomad bootstrap failed, just run more times?
DONE: ping doesn't work
[root@server-f:~]# ping 10.17.0.10
check reverse path firewall thingy.

consul fails to bootstrap, that's why nomad fails bootstrap as well

Can't pull from docker registry
```
[root@server-d:/etc/docker]# cat /nix/store/gprnayvnqss1a911ivaxjk6z82vddfwq-daemon.json
{
  "group": "docker",
  "hosts": [
    "fd://"
  ],
  "live-restore": true,
  "log-driver": "journald",
  "registry-mirrors": [
    "http://10.17.0.1:12778",
    "http://epl-docker-registry.service.consul:5000"
  ]
}
```

DONE: Fix gateways with masquerading
Just added route to everyones local gateways if nodes need to access nix-serve and docker registry

DONE: all neighbors are only in DC router subnet, shouldn't we prefer using unicast for available neighbors?
```
neighbor 192.168.12.10
neighbor 192.168.12.11
neighbor 192.168.12.13
```
DONE: keepalived cat state of both nodes to one for consul vrrp load balancing
DONE: test internet routing without default gw per subnet on nodes
DONE: integration tests to make sure reachability from subnet to all dc subnets
DONE: add internet routes for all nodes, part of nix plan to add in other places?
DONE: advertise 0.0.0.0 route in ospf, route is propogated but not working?
DONE: integration tests to make sure all nodes reach internet
DONE: bind works only after restart in multidc setup
DONE: write test that cross dc traffic is not masquerading?
TODO: move server firewall from l1 codegen into projection as data to be used in terraform as well?

* 2023-12-31

Written iperf stuff to disable masquerading, boom

* 2024-01-01

TODO: inside codegen have step of adding all of the changes to git
DONE: if terraform fails command doesn't fail
TODO: remove ssh interface column from server, just assume if server has public ip that is ssh, and if it doesn't assume LAN as we have VPN
DONE: AWS NixOS firewall issue

* 2024-01-02

To kill aws bootstrap route service:
```
ps aux | grep 'seq 1 3600' | grep -v grep | awk '{print $2}' | xargs -r kill
```

TODO: some weird state in makefile, wait-until-integration-tests fails with timeout inside the same process, when make is run again it works?
DONE: keepalived states are not generated in /var/run/
keepalived states only show current instance state, not another, effectively useless.

Next up: aws multiple subnet topology

Our routes are only good for internet for the private nodes, done through vrrp

DONE: aws internet interfaces should be marked as void/32 and dcrouter interface always eth1
DONE: aws public subnet bootstrap routes to the internal nodes
DONE: bootstrap internet routing for private nodes that don't have access to internet gateway, minimum static OSPF?

* 2024-01-07

DONE: bootstrap router node internet, which have router ip but no public ip
DONE: bootstrap private node internet, every subnet goes to its own private/vpn gateways
DONE: ospf routes don't advertise intradc routes for aws
DONE: vrrp switch doesn't work... keepalived can't execute it, no errors, when running switch by hand everything works...

* 2024-01-08

DONE: execute wait l1 provisioning loop from admin side for more reliability because connection can drop if loop is running remotely

DONE: aws single dc multisub there's no internet for server-e if it uses consul vrrp gateway server-b
DONE: fixed aws magic route
DONE: stash remaining changes, run integration tests and update test suites

* 2024-01-09

TODO: test that execute remote ssh command fails...
DONE: freeze aws multi dc test case

* 2024-01-11

DONE: freeze gcloud-single-dc environment
DONE: run gcloud multi dc environment

* 2024-01-12

DONE: freeze gcloud multi dc environment
DONE: gcloud multi subnet terraform
DONE: freeze gcloud multi subnet terraform

Google cloud has horrible networking, egress is throttled, even integration tests can't pass reasonably fast

* 2024-01-13

DONE: connect google cloud two dcs
DONE: server-a doesn't forward packet to dc3 server-e

udp packet travels from server-e to server-g but it doesn't travel back
seems that packet is halted at server-a, there's no firewall rule to drop it, supposed to go through wireguard interface
pings work from server-a to server-e directly, but forwarding from other server through server-a to server-e don't work.
it can't be aws firewall, because pings work back and forth from both nodes

Test udp datagram listen
```
nc -u -l 0.0.0.0 12345
```
Send datagram
```
echo lol | nc -u -w 0 -p 54321 10.17.0.12 12345
```

ping works from `10.17.0.10` server-a to `10.19.0.12`

there's no lol packet received in datagrams!!
tcp echo works, udp echo doesn't work from 10.17.0.12 to 10.17.0.10!!
must be gcloud firewall!!


Whoa... Google cloud is broken, it drops packets from 10.17.0.12 to 10.17.0.10 just because they're destined to 10.19.0.10!! I have to create google cloud router appliance and bgp peer with their faggotry cloud router which is worthless and brings no value!! That makes me consider dropping google cloud completely, useless google monkey faggot imbeciles cloud...

Maybe try virtio?

No packets received back when pinging from 10.19.0.10 to 10.17.0.12, it is only one way, packet last seen at 10.17.0.10 node

TODO: reserve first 5 ips and last 5 ips in subnet

https://en.wikibooks.org/wiki/Linux_Networking/IPIP_Encapsulation

IPIP just to do hops around google cloud fags?

DONE: after apply EPL_POSTROUTING is gone, poll every 10 seconds

* 2024-01-15

Direct hopping, we must establish GRE encapsulation in google cloud, there's about 20% performance hit, which is fine as we can't sature internet link anyway

1. DONE Add flag that all nodes that don't have access to VPN must teardown/setup remote hop in dc implementation
2. DONE Change VRRP script to instead setup gre tunnels on leaf nodes directly to candidate VPN node
3. DONE Change VRRP script to add route to the wg node
4. DONE Decide on gre private ip scheme, just use 10.17.128.0/16 subnet subnet for leaf nodes? yeah, that cuts ips in half but google cloud is horrible anyway and we'll recommend not to use it.
DONE: restrict every datacenter ips to not have 10.17.128.0/16 ips as those are reserved
5. DONE Add gre ip interfaces to trusted interfaces
6. DONE OSPF VPN interface sets up its gre link and adds all neighbors,
   1. we create link if it doesn't exists
   2. we add all neighbors (all leaf nodes)
   3. profit
7. DONE Add gre linux kernel modules!


Setup made:
gre tunnel server-a
```
[root@server-a:~]# cat setup
#!/bin/sh

ip tunnel add overNet mode gre local 10.17.0.10 key 1234
ip addr add 10.0.0.1/24 dev overNet

ip neighbor add 10.0.0.2 lladdr 10.17.0.12 dev overNet
ip link set dev overNet up
```

gre tunnel server-g
```
[root@server-g:~]# cat setup
#!/bin/sh

ip tunnel add overNet mode gre local 10.17.0.12 key 1234
ip addr add 10.0.0.2/24 dev overNet

ip neighbor add 10.0.0.1 lladdr 10.17.0.10 dev overNet
ip link set dev overNet up
```

Route change
```
[root@server-g:~]# ip route del 10.19.0.0/16
[root@server-g:~]# ip route add 10.19.0.0/16 src 10.17.0.12 via 10.0.0.1
```

Add overNet to trusted interfaces, profit!

* 2024-01-17

Implemented gre tunnels for google cloud

DONE: fix integration tests
DONE: freeze correctness in simulation test for gcloud-aws-multi-dc impl

* 2024-01-19

ping from 10.19.0.12 to 10.17.0.12 doesn't work.
ping from 10.17.0.12 to 10.19.0.12 works.
src is wrong for route '10.19.0.0/16 via 10.17.128.10 dev vpnGre' from google cloud server
We should use vpn gateway only as transport... do wireshark analysis?

Forward echo udp from 10.17.0.12 to 10.19.0.10 works
Reverse doesn't work
Packet should be routed through default gateway to our node, what the hell?

How about IP math at the edge, if packet is incoming from external network to VPN gateway:
1. translate destination address from 10.17.0.12 to 10.17.128.12
2. if packet is leaving the VPN gateway, translate source address from 10.17.128.12 to 10.17.0.12

* 2024-01-20

Server address translation
```
[root@server-g:~]# cat mookie
#!/bin/sh

# translate source address back
nft 'add rule ip nat PREROUTING ip daddr 10.17.128.12 ip daddr set 10.17.0.12'
```

Gateway address translation
```
[root@server-a:~]# cat mookie
#!/bin/sh

# into this subnet rule
nft 'add rule ip nat PREROUTING ip daddr 10.17.0.12 ip daddr set 10.17.128.12'
# outside this subnet rule
nft 'add rule ip nat PREROUTING ip saddr 10.17.128.12 ip saddr set 10.17.0.12'
```

Server address translation inside non gateway node
```
table ip INTER_DC_HOP_TRANSLATION {
  chain PREROUTING {
    type filter hook prerouting priority -300; policy accept;
    ip daddr 10.17.128.12 ip daddr set 10.17.0.12 counter return;
  }
}
```

Gateway hop address translation
```
table ip INTER_DC_HOP_TRANSLATION {
  chain PREROUTING {
    type filter hook prerouting priority -300; policy accept;
    # into this subnet rule
    ip daddr 10.17.0.12 ip saddr 10.0.0.0/8 ip daddr set 10.17.128.12
    # outside this subnet rule
    ip saddr 10.17.128.12 ip saddr set 10.17.0.12
  }
}
```

DONE: remove EPL_POSTROUTING chain and have separate table?
DONE: move to appropriate config of nftables without hacks https://github.com/NixOS/nixpkgs/pull/207758/files
DONE: nftables EPL_POSTROUTING chain doesn't need separate script, could be in `script` variable
DONE: upgrade nixos to 23.11, only then work on nftables stuff!!
DONE: iperf from 10.17.0.12 to 10.19.0.12 doesn't work!!

Only one echo goes from 10.19.0.12 to 10.17.0.12 !!! Nat sessions? Added snat from source ip, we still need ip translation rules

DONE: freeze gcloud-aws-multi-dc testcase in simulation
TODO: write script for testing all of the environments for CI? run all project compilations at once?

* 2024-01-24

Added global settings table
DONE: remove admin ssh key checking, we already have default root key
DONE: refactor out global settings table
DONE: move gcloud project id, gcloud bucket name, aws s3 bucket name to global_settings table
TODO: machine type reform
TODO: docker images reform

* 2024-01-25

TODO: check that all aws machine types start with aws.
TODO: check that all gcloud machine types start with gcloud.
DONE: EdenDB bug, if default and detached default is defined assertion panics, should be nice user error
DONE: Refactor all the manual tests for server reform with these changes:
1. delete server.cores detached default
2. delete server.memory_bytes detached default
3. add `
    datacenter.default_server_kind testvm.cpu4ram8192,
  datacenter.default_server_kind testvm.cpu2ram8192,
  datacenter.default_server_kind testvm.cpu2ram2048,
` detached default
4. add server kind data for detached default
```
DATA STRUCT server_kind {
    kind: testvm.cpu4ram8192,
    cores: 4,
    memory_bytes: 8589934592,
    reserved_memory_bytes: 134217728,
    architecture: x86_64,
}
```

* 2024-01-26

DONE: test data, include google cloud? Or disable assertion when testing? Always include data might be right answer...

TODO: automate renewal of aws availability zones
TODO: automate renewal of gcloud availability zones
DONE: automate renewal of gcloud machine types
DONE: automate renewal of aws machine types

* 2024-01-27

DONE: restore build.rs
DONE: add two tests for aws, defining custom type + wrong server kind
TODO: arm support, should be quite easy? start with single dc
TODO: arm support: build NixOS vm images for clouds for all architectures
DONE: server spec preconditions
DONE: redo nftables function as allowed by new NixOS

TODO: fix nixos generate arm error when running `make vm-images` inside single-dc-warm

https://www.reddit.com/r/NixOS/comments/eiyfsl/nixops_aarch64_targets/

```
error: a 'aarch64-linux' with features {} is required to build '/nix/store/77g213ha4f0schkfa3wwgvjy629z1dbf-nixos-enter.drv', but I am a 'x86_64-linux' with features {benchmark, big-parallel, kvm, nixos-test}
make: *** [Makefile:112: servers/vm-template-arm64.txt] Error 1
```

TODO: add check about cross compiling, how do we check in Makefile that system supports that?
```
{
  # Enable binfmt emulation of aarch64-linux.
  boot.binfmt.emulatedSystems = [ "aarch64-linux" ];
}
```

TODO: build multiplatform docker images for all userspace apps
TODO: our docker image hashes break now!!! we need hashes for all architectures.
TODO: add system labeling to run certain types of workloads
DONE: when cloning envs we must copy all the app sources also

* 2024-02-01

Arm unreasonable to run inside vms, do inside google cloud

* 2024-02-02

DONE: add inside preconditions architecture
TODO: don't build images for unneeded clouds, say, any cloud needs arm don't build for all clouds

* 2024-02-03

DONE: add remaining aws arm machines in jq
TODO: arm support either wait for crosscompilation to work or support arm builder
DONE: move arm support to experimental flag disabled by default

arm fails in google cloud, tried rebuilding image many ways, error also described here https://cloud.google.com/compute/docs/troubleshooting/troubleshooting-arm-vms#arm_vm_instance_doesnt_boot
resolution: let NixOS mature to support arm but it is work in progress now https://nixos.wiki/wiki/NixOS_on_ARM
`ARM support for NixOS is a work-in-progress, but is progressing quickly.`

```
BdsDxe: failed to load Boot0001 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x2,0x0)/NVMe(0x1,00-00-00-00-00-00-00-00): Not Found
UEFI firmware (version  built at 13:52:42 on Nov 17 2023)
EMU Variable FVB Started
EMU Variable invalid PCD sizes
Found PL031 RTC @ 0x9010000
InitializeRealTimeClock: using default timezone/daylight settings
[2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
UEFI: Failed to load image.
Description: UEFI Misc Device
FilePath: PciRoot(0x0)/Pci(0x2,0x0)/NVMe(0x1,00-00-00-00-00-00-00-00)
OptionNumber: 1.
Status: Not Found.
```

* 2024-02-09

TODO: standardize docker images for jobs
DONE: write check that docker image pin consists of only one image per architecture
TODO: refactor fetch task in nomad to accept docker image as argument (maybe pin?)
TODO: what to do with images we'd want to build with nix and add?
TODO: can we retag images for region and cache in our infra so that after bootstrap we pull into our minio?

Now just propogate images to jobs.
We could wrap for nomad task a type that either contains pointer to epl image or epl application deployment pointer and both are just turned into strings, boom!

TODO: write check that bound nomad job docker image architecture always matches the host
DONE: ensure that arm64 in nomad is arm64 from sources

* 2024-02-11

DONE: write test for DockerImageDoesNotBelongToTheExpectedSet
DONE: write test for DockerImageNotFoundForArchitectureForPin
TODO: error out if arm machines are not available but arm deployments exist
DONE: think of what to do with deployment and image architectures... frontend doesn't matter, only backend could be arm or amd64. We pick architecture in the app deployment and it should be built for both in flakes?
DONE: write assert that all groups have architecture constraint?
TODO: run tests with the oldest postgres database image used in the infra?

NEXT: server labels and improve memory model according to specific placements

DONE: generate nomad labels for placements
DONE: remove arch label, we don't need it, irrelevant, job already decides workload architecture?
DONE: use placement queries for subtracting memory

DONE: add placements to all stateless workloads
- frontend apps
- backend apps
- grafana

NEXT: admin panel credentials
NEXT: dynamic nginx reload
NEXT: auto ssl certificates

2024-02-12

DONE: add admin panel password to integration tests

```
    generated::admin_panel_responds_responds
    generated::consul_ui_external_responds_us_west
    generated::grafana_external_admin_panel_responds_main
    generated::grafana_node_exporter_dashboard_loaded_main
    generated::loki_cluster_main_has_l1_provisioning_stream
    generated::minio_external_admin_panel_responds_global
    generated::monitoring_cluster_external_admin_panel_responds_default
    generated::monitoring_cluster_external_admin_panel_responds_secondary
    generated::nomad_ui_external_responds_us_west
    generated::vault_ui_external_responds_us_west
    manual::exposed_epl_app::epl_app_prometheus_metrics_gathered
```

2024-02-13

NEXT: root on zfs
So this builds relase of nixos image with zfs for aws inside nixpkgs
```
nix-build nixos/release.nix -A amazonImageZfs.x86_64-linux

```

Other that works
```
nix-build '<nixpkgs/nixos/release.nix>' \
    -A amazonImageZfs.x86_64-linux \
    --arg configuration ./amazon.nix \
    -o ./result
```

So zfs image is built in root like that
```
  amazonImageZfs = forMatchingSystems [ "x86_64-linux" "aarch64-linux" ] (system:

    with import ./.. { inherit system; };

    hydraJob ((import lib/eval-config.nix {
      inherit system;
      modules =
        [ configuration
          versionModule
          ./maintainers/scripts/ec2/amazon-image-zfs.nix
        ];
    }).config.system.build.amazonImage)

  );
```

amazon-image-zfs.nix just a config, soooo...
all the logic is in amazon-image.nix?
```
{
  imports = [ ./amazon-image.nix ];
  ec2.zfs = {
    enable = true;
    datasets = {
      "tank/system/root".mount = "/";
      "tank/system/var".mount = "/var";
      "tank/local/nix".mount = "/nix";
      "tank/user/home".mount = "/home";
    };
  };
}
```

amazon-image.nix has advanced zfs builder which is chosen, copy this!
```
    zfsBuilder = import ../../../lib/make-multi-disk-zfs-image.nix {
      inherit lib config configFile;
      inherit (cfg) contents format name;
      pkgs = import ../../../.. { inherit (pkgs) system; }; # ensure we use the regular qemu-kvm package

      includeChannel = true;

      bootSize = 1000; # 1G is the minimum EBS volume

      rootSize = cfg.sizeMB;
      rootPoolProperties = {
        ashift = 12;
        autoexpand = "on";
      };

      datasets = config.ec2.zfs.datasets;

      postVM = ''
        extension=''${rootDiskImage##*.}
        friendlyName=$out/${cfg.name}
        rootDisk="$friendlyName.root.$extension"
        bootDisk="$friendlyName.boot.$extension"
        mv "$rootDiskImage" "$rootDisk"
        mv "$bootDiskImage" "$bootDisk"

        mkdir -p $out/nix-support
        echo "file ${cfg.format} $bootDisk" >> $out/nix-support/hydra-build-products
        echo "file ${cfg.format} $rootDisk" >> $out/nix-support/hydra-build-products

       ${pkgs.jq}/bin/jq -n \
         --arg system_label ${lib.escapeShellArg config.system.nixos.label} \
         --arg system ${lib.escapeShellArg pkgs.stdenv.hostPlatform.system} \
         --arg root_logical_bytes "$(${pkgs.qemu_kvm}/bin/qemu-img info --output json "$rootDisk" | ${pkgs.jq}/bin/jq '."virtual-size"')" \
         --arg boot_logical_bytes "$(${pkgs.qemu_kvm}/bin/qemu-img info --output json "$bootDisk" | ${pkgs.jq}/bin/jq '."virtual-size"')" \
         --arg boot_mode "${amiBootMode}" \
         --arg root "$rootDisk" \
         --arg boot "$bootDisk" \
        '{}
          | .label = $system_label
          | .boot_mode = $boot_mode
          | .system = $system
          | .disks.boot.logical_bytes = $boot_logical_bytes
          | .disks.boot.file = $boot
          | .disks.root.logical_bytes = $root_logical_bytes
          | .disks.root.file = $root
          ' > $out/nix-support/image-info.json
      '';
    };

```

this make multi disk zfs image is interesting also
```
nixpkgs git:(master) find . | grep make-multi-disk-zfs-image
./nixos/lib/make-multi-disk-zfs-image.nix
```

What is the conclusion? Gut and extract the following
```
import lib/eval-config.nix {
      inherit system;
      modules =
        [ configuration
          versionModule
          ./maintainers/scripts/ec2/amazon-image-zfs.nix
        ];
    }
```

Then do the same nix-build and should work?

This is how we call eval-config.nix
```
nix-build '<nixpkgs/nixos/lib/eval-config.nix>' \
   -A config.system.build.googleComputeImage \
   --arg modules "[ <nixpkgs/nixos/modules/virtualisation/google-compute-image.nix> ]" \
   --arg config "{ inherit (config.virtualization.googleComputeImage); configFile = \"./mcpezzlow2.nix\"; }"  \
   --argstr system x86_64-linux \
   -o gce \
   -j 10
```

2024-02-14

DONE: fork nixos-generators and add qcow-zfs and the rest of the images

disk config, reflect this

DONE: make cargo.lock updates from the makefile, only update lockfile if compile env was updated, boom
DONE: check if user/group exists for secrets and they can be instantiated now instead of later
DONE: zfs on root aws image
TODO: detect image push failure in l2 provisioning
DONE: change server_components to l2_components

2024-02-15

DONE: aws has space but doesn't expand root zpool

zpool expansion on aws:
```
ZFS_DEVICE=$( readlink -f /dev/disk/by-label/tank | sed -E 's/p[0-9]$//' )
ZFS_PARTITION_NO=$( readlink -f /dev/disk/by-label/tank | sed -E 's/^.*p([0-9])$/\1/' )
if [ -f "${ZFS_DEVICE}9" ];
then
    parted -s $ZFS_DEVICE rm 9 || true
    growpart $ZFS_DEVICE $ZFS_PARTITION_NO
    # last step
    zpool online -e tank $ZFS_DEVICE
fi
```

DONE: no sed? cmon!! absolute paths? (unneeded)
```
/nix/store/1fwkw687l65zap3lf4ph3k2dgr1hqsnl-nixos-system-server-c-23.11.20231129.057f9ae/activate: line 196: sed: command not found
readlink: write error: Broken pipe
/nix/store/1fwkw687l65zap3lf4ph3k2dgr1hqsnl-nixos-system-server-c-23.11.20231129.057f9ae/activate: line 197: sed: command not found
readlink: write error: Broken pipe
```

DONE: separate ebs volume attachments (unneeded)

DONE: expand zfs partition ourselves and use only one zfs disk image, too much sacrifice for autoexpand and too many issues...

the only win for multi disk zfs is extra disk space, other than that multi disk is cancer.

DONE: add precondition that root is on zfs
DONE: implement expand on boot from provisioning

2024-02-16

DONE: prevent server on running stateless workloads

TODO:

docs, define l1 provisioning
docs, define l1 provisioning secrets
docs, define l2 provisioning
docs, define l2 provisioning secrets
docs, define l2 provisioning l1 secret templates
docs, server runtime

DONE: development docs, add new cloud, we must have nixos with zfs on root image. image must reboot successfully after provisioning (we must not damage grub settings with l1 TODO)

create: zfs dataset provisioning for consul/nomad
DONE: vault: move to internal storage engine separated from consul?
DONE: all root volumes zfs datasets

TODO: test vms image generation is different from terraform, terraform does nix build derivations which are more flexible but test vms use command line nixos-generate
DONE: fix vault unsealing for raft consensus. Do vault operator init as a separate step per cluster. Then with the results unseal all nodes.
DONE: add precondition that the swap is disabled on server
TODO: maybe remove modules/config paths from vm build and make it depend only on pinned nixpkgs?
DONE: fucking bind doesn't work on first provisioning, motherfuckers and time wasters
DONE: dnsseckeys not propogated to server-b... you fucking piece of shit... they suffer from same problem as l2 key propogation... activation happens before users are created?..
DONE: move cp -pu dns sec keys to post l2 secrets step

```
activating the configuration...
id: ‘named’: no such user
id: ‘named’: no such user
Activation script snippet 'bindActivation' failed (1)
```

NEXT: create encrypted zfs datasets for consul/vault/nomad

2024-02-20

DONE: small benchmarking tool to see timings

2024-02-21

DONE: zfs exporter
TODO: zfs backups
TODO: bind during boot /run/current-system/sw/bin directory is not available, so created dns files are empty!! use absolute paths in pkgs?
DONE: admin user for users to login
DONE: zfs disk space usage visualization
DONE: zfs biggest datasets

2024-02-22

DONE: custom zpool provisioning
DONE: XFS jbod provisioning
DONE: disk volume naming reform
DONE: region peering (nomad + consul)

Works to build server configuration:
```
nix-build '<nixpkgs/nixos>' -A config.system.build.toplevel -I nixos-config=./configuration.nix
```

DONE: add l1 build all script to makefile to evaluate on test vms
TODO: monitor container counts
DONE: memory simulator places two loki readers on the same server, bad, they should be spread across servers! Do memory groups?
DONE: force nomad label to allow running unassigned workloads

2024-02-23

DONE: correct epl system reserved memory of 1.0GB for nomad + consul
DONE: add precondition for root disk name

this gets one disk, what if grep gives more than one?
zpool status -P rpool | grep '/dev/' | awk '{print $1}' | sed -E 's/-part[0-9]+$//' | xargs readlink -f

next we check that root disk belongs to tank?

DONE: add detached default of disk kind

2024-02-25

DONE: disk creation for qemu
DONE: disk creation for terraform - google cloud
DONE: disk creation for terraform - aws
DONE: aws disk image build broken, possibly google cloud too?

ZFS:
DONE: check vdev disk uniqueness
DONE: check vdev spare disk uniqueness
DONE: check disk capacity inside dev is all equal
DONE: write all zfs disk analsysis tests for every error


2024-02-25

DONE: aws generate all medium types
DONE: force that all disks in aws are of existing aws disk types
DONE: gcloud generate all medium types
DONE: force that all disks in gcloud are of existing gcloud disk types
TODO: unify vm-template.nix with cloud provisioning nix files
DONE: what to do with elastic disk sizing? it should be treated specially?
DONE: create xfs jbod volumes
DONE: register those in nomad
TODO: l1 provisioning chmod wrong log when executing other l1 provisioning in parallel
TODO: build with nix-build all user apps at once

2024-02-27

DONE: check that zfs cannot use more than one vdev if it uses elastic disks
DONE: provision zpools
DONE: support zpool slog, must have same sizes
DONE: support zpool cache
TODO: support serials on disks and enforce zpool creation either with serials or devs

2024-02-29

DONE: move out all activation scripts to l1 provisioning post/pre hooks?

2024-03-01

DONE: aws disk errors
DONE: aws error tests
DONE: aws root disk terraform
DONE: add optional minimum capacity with checks
DONE: aws other disks terraform
DONE: aws st1 and sc1 cannot be boot volumes!
DONE: check that aws extra disk names must start with `sd[f-p]`
DONE: disks are provisioned in aws, but preconditions now fail!
AWS recommends sd[f-p] naming... symlink exists for that. lsblk now shows
wrong drives, wtf... how could we solve this?.. everything now should be nvme?..
there should be aws name and actual name?.. oy vey...
we could derive mappings of disks in script?.. ......

2024-03-02

metal disk layout, same as nitro
DONE: run aws tests and move to gcloud
```
[root@ip-10-17-0-13:~]# lsblk 
NAME        MAJ:MIN RM  SIZE RO TYPE MOUNTPOINTS
nvme0n1     259:0    0   20G  0 disk 
├─nvme0n1p1 259:1    0    1M  0 part 
├─nvme0n1p2 259:2    0  998M  0 part /boot
└─nvme0n1p3 259:3    0   19G  0 part 
nvme1n1     259:4    0   20G  0 disk

[root@ip-10-17-0-13:~]# ls -lha /dev/ | grep sdf
lrwxrwxrwx  1 root root           7 Mar  2 09:29 sdf -> nvme1n1

[root@ip-10-17-0-13:~]# ls -lha /dev/ | grep nvme0
crw-------  1 root root    249,   0 Mar  2 09:28 nvme0
brw-rw----  1 root disk    259,   0 Mar  2 09:28 nvme0n1
brw-rw----  1 root disk    259,   1 Mar  2 09:28 nvme0n1p1
brw-rw----  1 root disk    259,   2 Mar  2 09:28 nvme0n1p2
brw-rw----  1 root disk    259,   3 Mar  2 09:28 nvme0n1p3
lrwxrwxrwx  1 root root           7 Mar  2 09:28 xvda -> nvme0n1
lrwxrwxrwx  1 root root           9 Mar  2 09:28 xvda1 -> nvme0n1p1
lrwxrwxrwx  1 root root           9 Mar  2 09:28 xvda2 -> nvme0n1p2
lrwxrwxrwx  1 root root           9 Mar  2 09:28 xvda3 -> nvme0n1p3

[root@ip-10-17-0-13:~]# ls -lha /dev/
total 8.5K
drwxr-xr-x 16 root root        3.4K Mar  2 09:29 .
drwxr-xr-x 16 root root          16 Feb 17 06:26 ..
crw-r--r--  1 root root     10, 235 Mar  2 09:28 autofs
drwxr-xr-x  2 root root         300 Mar  2 09:29 block
crw-------  1 root root     10, 234 Mar  2 09:28 btrfs-control
drwxr-xr-x  3 root root          60 Mar  2 09:28 bus
drwxr-xr-x  2 root root        2.5K Mar  2 09:29 char
crw--w----  1 root tty       5,   1 Mar  2 09:28 console
lrwxrwxrwx  1 root root          11 Mar  2 09:28 core -> /proc/kcore
crw-------  1 root root     10, 125 Mar  2 09:28 cpu_dma_latency
crw-------  1 root root     10, 203 Mar  2 09:28 cuse
drwxr-xr-x  9 root root         180 Mar  2 09:28 disk
lrwxrwxrwx  1 root root          13 Mar  2 09:28 fd -> /proc/self/fd
crw-rw-rw-  1 root root      1,   7 Mar  2 09:28 full
crw-rw-rw-  1 root root     10, 229 Mar  2 09:28 fuse
crw-------  1 root root     10, 228 Mar  2 09:28 hpet
drwxr-xr-x  2 root root           0 Mar  2 09:28 hugepages
crw-------  1 root root     10, 183 Mar  2 09:28 hwrng
drwxr-xr-x  2 root root          60 Mar  2 09:28 input
crw-r--r--  1 root root      1,  11 Mar  2 09:28 kmsg
crw-rw-rw-  1 root kvm      10, 232 Mar  2 09:28 kvm
lrwxrwxrwx  1 root root          28 Mar  2 09:28 log -> /run/systemd/journal/dev-log
brw-rw----  1 root disk      7,   0 Mar  2 09:28 loop0
brw-rw----  1 root disk      7,   1 Mar  2 09:28 loop1
brw-rw----  1 root disk      7,   2 Mar  2 09:28 loop2
brw-rw----  1 root disk      7,   3 Mar  2 09:28 loop3
brw-rw----  1 root disk      7,   4 Mar  2 09:28 loop4
brw-rw----  1 root disk      7,   5 Mar  2 09:28 loop5
brw-rw----  1 root disk      7,   6 Mar  2 09:28 loop6
brw-rw----  1 root disk      7,   7 Mar  2 09:28 loop7
crw-rw----  1 root disk     10, 237 Mar  2 09:28 loop-control
drwxr-xr-x  2 root root          60 Mar  2 09:28 mapper
drwxr-xr-x  2 root root          40 Mar  2 09:28 .mdadm
crw-r-----  1 root kmem      1,   1 Mar  2 09:28 mem
drwxrwxrwt  2 root root          40 Mar  2 09:28 mqueue
crw-------  1 root root     90,   0 Mar  2 09:28 mtd0
crw-------  1 root root     90,   1 Mar  2 09:28 mtd0ro
drwxr-xr-x  2 root root          60 Mar  2 09:28 net
crw-------  1 root root    248,   0 Mar  2 09:28 ng0n1
crw-------  1 root root    248,   1 Mar  2 09:29 ng1n1
crw-rw-rw-  1 root root      1,   3 Mar  2 09:28 null
crw-------  1 root root    249,   0 Mar  2 09:28 nvme0
brw-rw----  1 root disk    259,   0 Mar  2 09:28 nvme0n1
brw-rw----  1 root disk    259,   1 Mar  2 09:28 nvme0n1p1
brw-rw----  1 root disk    259,   2 Mar  2 09:28 nvme0n1p2
brw-rw----  1 root disk    259,   3 Mar  2 09:28 nvme0n1p3
crw-------  1 root root    249,   1 Mar  2 09:29 nvme1
brw-rw----  1 root disk    259,   4 Mar  2 09:29 nvme1n1
crw-------  1 root root     10, 144 Mar  2 09:28 nvram
crw-r-----  1 root kmem      1,   4 Mar  2 09:28 port
crw-------  1 root root    108,   0 Mar  2 09:28 ppp
crw-rw-rw-  1 root tty       5,   2 Mar  2  2024 ptmx
drwxr-xr-x  2 root root           0 Mar  2 09:28 pts
crw-rw-rw-  1 root root      1,   8 Mar  2 09:28 random
crw-rw-r--  1 root root     10, 242 Mar  2 09:28 rfkill
lrwxrwxrwx  1 root root           4 Mar  2 09:28 rtc -> rtc0
crw-------  1 root root    253,   0 Mar  2 09:28 rtc0
lrwxrwxrwx  1 root root           7 Mar  2 09:29 sdf -> nvme1n1
drwxrwxrwt  2 root root          40 Mar  2 09:28 shm
crw-------  1 root root     10, 231 Mar  2 09:28 snapshot
drwxr-xr-x  2 root root          80 Mar  2 09:28 snd
lrwxrwxrwx  1 root root          15 Mar  2 09:28 stderr -> /proc/self/fd/2
lrwxrwxrwx  1 root root          15 Mar  2 09:28 stdin -> /proc/self/fd/0
lrwxrwxrwx  1 root root          15 Mar  2 09:28 stdout -> /proc/self/fd/1
crw-rw-rw-  1 root tty       5,   0 Mar  2 09:28 tty
crw--w----  1 root tty       4,   0 Mar  2 09:28 tty0
crw--w----  1 root tty       4,   1 Mar  2 09:28 tty1
crw--w----  1 root tty       4,  10 Mar  2 09:28 tty10
crw--w----  1 root tty       4,  11 Mar  2 09:28 tty11
crw--w----  1 root tty       4,  12 Mar  2 09:28 tty12
crw--w----  1 root tty       4,  13 Mar  2 09:28 tty13
crw--w----  1 root tty       4,  14 Mar  2 09:28 tty14
crw--w----  1 root tty       4,  15 Mar  2 09:28 tty15
crw--w----  1 root tty       4,  16 Mar  2 09:28 tty16
crw--w----  1 root tty       4,  17 Mar  2 09:28 tty17
crw--w----  1 root tty       4,  18 Mar  2 09:28 tty18
crw--w----  1 root tty       4,  19 Mar  2 09:28 tty19
crw--w----  1 root tty       4,   2 Mar  2 09:28 tty2
crw--w----  1 root tty       4,  20 Mar  2 09:28 tty20
crw--w----  1 root tty       4,  21 Mar  2 09:28 tty21
crw--w----  1 root tty       4,  22 Mar  2 09:28 tty22
crw--w----  1 root tty       4,  23 Mar  2 09:28 tty23
crw--w----  1 root tty       4,  24 Mar  2 09:28 tty24
crw--w----  1 root tty       4,  25 Mar  2 09:28 tty25
crw--w----  1 root tty       4,  26 Mar  2 09:28 tty26
crw--w----  1 root tty       4,  27 Mar  2 09:28 tty27
crw--w----  1 root tty       4,  28 Mar  2 09:28 tty28
crw--w----  1 root tty       4,  29 Mar  2 09:28 tty29
crw--w----  1 root tty       4,   3 Mar  2 09:28 tty3
crw--w----  1 root tty       4,  30 Mar  2 09:28 tty30
crw--w----  1 root tty       4,  31 Mar  2 09:28 tty31
crw--w----  1 root tty       4,  32 Mar  2 09:28 tty32
crw--w----  1 root tty       4,  33 Mar  2 09:28 tty33
crw--w----  1 root tty       4,  34 Mar  2 09:28 tty34
crw--w----  1 root tty       4,  35 Mar  2 09:28 tty35
crw--w----  1 root tty       4,  36 Mar  2 09:28 tty36
crw--w----  1 root tty       4,  37 Mar  2 09:28 tty37
crw--w----  1 root tty       4,  38 Mar  2 09:28 tty38
crw--w----  1 root tty       4,  39 Mar  2 09:28 tty39
crw--w----  1 root tty       4,   4 Mar  2 09:28 tty4
crw--w----  1 root tty       4,  40 Mar  2 09:28 tty40
crw--w----  1 root tty       4,  41 Mar  2 09:28 tty41
crw--w----  1 root tty       4,  42 Mar  2 09:28 tty42
crw--w----  1 root tty       4,  43 Mar  2 09:28 tty43
crw--w----  1 root tty       4,  44 Mar  2 09:28 tty44
crw--w----  1 root tty       4,  45 Mar  2 09:28 tty45
crw--w----  1 root tty       4,  46 Mar  2 09:28 tty46
crw--w----  1 root tty       4,  47 Mar  2 09:28 tty47
crw--w----  1 root tty       4,  48 Mar  2 09:28 tty48
crw--w----  1 root tty       4,  49 Mar  2 09:28 tty49
crw--w----  1 root tty       4,   5 Mar  2 09:28 tty5
crw--w----  1 root tty       4,  50 Mar  2 09:28 tty50
crw--w----  1 root tty       4,  51 Mar  2 09:28 tty51
crw--w----  1 root tty       4,  52 Mar  2 09:28 tty52
crw--w----  1 root tty       4,  53 Mar  2 09:28 tty53
crw--w----  1 root tty       4,  54 Mar  2 09:28 tty54
crw--w----  1 root tty       4,  55 Mar  2 09:28 tty55
crw--w----  1 root tty       4,  56 Mar  2 09:28 tty56
crw--w----  1 root tty       4,  57 Mar  2 09:28 tty57
crw--w----  1 root tty       4,  58 Mar  2 09:28 tty58
crw--w----  1 root tty       4,  59 Mar  2 09:28 tty59
crw--w----  1 root tty       4,   6 Mar  2 09:28 tty6
crw--w----  1 root tty       4,  60 Mar  2 09:28 tty60
crw--w----  1 root tty       4,  61 Mar  2 09:28 tty61
crw--w----  1 root tty       4,  62 Mar  2 09:28 tty62
crw--w----  1 root tty       4,  63 Mar  2 09:28 tty63
crw--w----  1 root tty       4,   7 Mar  2 09:28 tty7
crw--w----  1 root tty       4,   8 Mar  2 09:28 tty8
crw--w----  1 root tty       4,   9 Mar  2 09:28 tty9
crw-rw----  1 root dialout   4,  64 Mar  2 09:28 ttyS0
crw-rw----  1 root dialout   4,  65 Mar  2 09:28 ttyS1
crw-rw----  1 root dialout   4,  66 Mar  2 09:28 ttyS2
crw-rw----  1 root dialout   4,  67 Mar  2 09:28 ttyS3
crw-------  1 root root     10, 239 Mar  2 09:28 uhid
crw-------  1 root root     10, 223 Mar  2 09:28 uinput
crw-rw-rw-  1 root root      1,   9 Mar  2 09:28 urandom
crw-------  1 root root     10, 126 Mar  2 09:28 userfaultfd
crw-------  1 root root     10, 240 Mar  2 09:28 userio
crw-rw----  1 root tty       7,   0 Mar  2 09:28 vcs
crw-rw----  1 root tty       7,   1 Mar  2 09:28 vcs1
crw-rw----  1 root tty       7,   2 Mar  2 09:28 vcs2
crw-rw----  1 root tty       7,   3 Mar  2 09:28 vcs3
crw-rw----  1 root tty       7,   4 Mar  2 09:28 vcs4
crw-rw----  1 root tty       7,   5 Mar  2 09:28 vcs5
crw-rw----  1 root tty       7,   6 Mar  2 09:28 vcs6
crw-rw----  1 root tty       7, 128 Mar  2 09:28 vcsa
crw-rw----  1 root tty       7, 129 Mar  2 09:28 vcsa1
crw-rw----  1 root tty       7, 130 Mar  2 09:28 vcsa2
crw-rw----  1 root tty       7, 131 Mar  2 09:28 vcsa3
crw-rw----  1 root tty       7, 132 Mar  2 09:28 vcsa4
crw-rw----  1 root tty       7, 133 Mar  2 09:28 vcsa5
crw-rw----  1 root tty       7, 134 Mar  2 09:28 vcsa6
crw-rw----  1 root tty       7,  64 Mar  2 09:28 vcsu
crw-rw----  1 root tty       7,  65 Mar  2 09:28 vcsu1
crw-rw----  1 root tty       7,  66 Mar  2 09:28 vcsu2
crw-rw----  1 root tty       7,  67 Mar  2 09:28 vcsu3
crw-rw----  1 root tty       7,  68 Mar  2 09:28 vcsu4
crw-rw----  1 root tty       7,  69 Mar  2 09:28 vcsu5
crw-rw----  1 root tty       7,  70 Mar  2 09:28 vcsu6
drwxr-xr-x  2 root root          60 Mar  2 09:28 vfio
crw-------  1 root root     10, 127 Mar  2 09:28 vga_arbiter
crw-------  1 root root     10, 137 Mar  2 09:28 vhci
crw-rw-rw-  1 root kvm      10, 238 Mar  2 09:28 vhost-net
crw-rw-rw-  1 root kvm      10, 241 Mar  2 09:28 vhost-vsock
crw-------  1 root root     10, 130 Mar  2 09:28 watchdog
crw-------  1 root root    243,   0 Mar  2 09:28 watchdog0
lrwxrwxrwx  1 root root           7 Mar  2 09:28 xvda -> nvme0n1
lrwxrwxrwx  1 root root           9 Mar  2 09:28 xvda1 -> nvme0n1p1
lrwxrwxrwx  1 root root           9 Mar  2 09:28 xvda2 -> nvme0n1p2
lrwxrwxrwx  1 root root           9 Mar  2 09:28 xvda3 -> nvme0n1p3
crw-rw-rw-  1 root root      1,   5 Mar  2 09:28 zero
crw-rw-rw-  1 root root     10, 249 Mar  2 09:28 zfs
```

xen disk layout, xvda now

```
[root@ip-10-17-0-12:~]# lsblk 
NAME    MAJ:MIN RM  SIZE RO TYPE MOUNTPOINTS
xvda    202:0    0   20G  0 disk 
├─xvda1 202:1    0    1M  0 part 
├─xvda2 202:2    0  998M  0 part /boot
└─xvda3 202:3    0   19G  0 part 
xvdf    202:80   0   20G  0 disk
[root@ip-10-17-0-12:~]# ls -lha /dev/ | grep sdf
lrwxrwxrwx  1 root root           4 Mar  2 09:29 sdf -> xvdf
[root@ip-10-17-0-12:~]# ls -lha /dev/ | grep xvda
lrwxrwxrwx  1 root root           4 Mar  2 09:28 sda -> xvda
lrwxrwxrwx  1 root root           5 Mar  2 09:28 sda1 -> xvda1
lrwxrwxrwx  1 root root           5 Mar  2 09:28 sda2 -> xvda2
lrwxrwxrwx  1 root root           5 Mar  2 09:28 sda3 -> xvda3
brw-rw----  1 root disk    202,   0 Mar  2 09:28 xvda
brw-rw----  1 root disk    202,   1 Mar  2 09:28 xvda1
brw-rw----  1 root disk    202,   2 Mar  2 09:28 xvda2
brw-rw----  1 root disk    202,   3 Mar  2 09:28 xvda3

```

2024-03-03

```
[root@ip-10-17-0-13:~]# free -b
               total        used        free      shared  buff/cache   available
Mem:     405366648832  2075807744 403190542336    12578816   100298752 401535631360
Swap:              0           0           0

```
405366648832
412316860416

Google faggot cloud attaches dev name sda disk in mixed order!......

DONE: use disk labels and have it as datacenter attribute...
DONE: change convention for google cloud to not be confusing when you're inside server... Use vda now instead of sda...
DONE: check no more than 256TB for all disks as per google docs


2024-03-04

DONE: changed preconditions
DONE: change zpool provisioning
DONE: change xfs provisioning
DONE: support google cloud pd-extreme disk iops option, know how it is calculated not to provision too much

Next: frontend load balancer consul live no downtime reload

2024-03-05

1. l2 provisioning can contain secrets, like nginx config, move l2 provisioning to /run/secdir
2. Allow secret to be a file
3. Metrics uses meta.private_ip -> move this out to original site
4. we use third party container for open resty... how about build with nix?
5. No rust library for gzip... gzip would be too slow if from command line...
6. Build our own load balancer with nix and zstd?

We can unbase64 script, gzip it, then base64 again!!

DONE: monitor when our compressed config is approaching 512KB of vault

added assert

2024-03-06

TODO: secret sharding for site.conf external lb for all site files to never approach 512KB
DONE: admin page html hot reload
DONE: consul federation

2024-03-07

DONE: setup hostnames on every node
DONE: setup admin account instead of ssh root@
TODO: reserve first 7 ips from subnet
DONE: add column in disk kinds has_extra_attrs, default to false and extra attrs must be empty for that
TODO: after consul upgrade to 1.18 split off dns ACL token from default token
DONE: vault changes for secrets is polling model, not realtime notification model. MOVE TO CONSUL.
DONE: move entire external lb nginx config to consul template

2024-03-08

TODO: upgrade vault secrets policies field in nomad jobs, will be removed in nomad 1.9
DONE: provision kv values in consul templates
TODO: dump external lb nginx config for reference

Done with aws disk IOPS provisioning, do same for gcloud

DONE: grow xfs filesystem automatically

2024-03-09

TODO: support gcloud hyperdisk, but our pd- disks are good enough now.
I'd need to juggle a lot of which machine supports what, whatever.

Default 60GB hyperdisk balanced options, default is 6 IOPS * 20 GB + 1.5 mb throughput * 20 GB
```
      ~ provisioned_iops          = 3120 -> 3000
      ~ provisioned_throughput    = 170 -> 140
```

```
│ Error: execution halted
│ 
│ Error: Error creating Disk: googleapi: Error 400: Requested provisioned IOPS is too high for the requested disk size., badRequest
│ 
│   with google_compute_disk.extra_disk_server-d_disk_vda,
│   on main.tf line 292, in resource "google_compute_disk" "extra_disk_server-d_disk_vda":
│  292: resource "google_compute_disk" "extra_disk_server-d_disk_vda" {
│ 
╵
╷
│ Error: Error waiting for Updating Disk: error while retrieving operation: unable to finish polling, context has been cancelled
│ 
│   with google_compute_disk.extra_disk_server-d_disk_vdb,
│   on main.tf line 312, in resource "google_compute_disk" "extra_disk_server-d_disk_vdb":
│  312: resource "google_compute_disk" "extra_disk_server-d_disk_vdb" {
│ 
╵
╷
╷
│ Error: googleapi: Error 400: hyperdisk-extreme disk type cannot be used by e2-standard-4 machine type., badRequest
│ 
│   with google_compute_attached_disk.extra_disk_server-d_disk_vdd,
│   on main.tf line 365, in resource "google_compute_attached_disk" "extra_disk_server-d_disk_vdd":
│  365: resource "google_compute_attached_disk" "extra_disk_server-d_disk_vdd" {
```

Baseline perf with no configs for pd-extreme disks on e2-standard-4 machine is 15k IOPS, which is decent

DONE: make sure we don't lose vault/nomad token after init

2024-03-10

DONE: fix the vault issue with booststrap (it wasn't vault but nomad?)

Nomad bootstrap error:
Error bootstrapping: Unexpected response code: 500 (No cluster leader)

TODO: add nix-collect-garbage periodic job

Tempo has scalable monolithic mode!!!
https://github.com/grafana/tempo/blob/main/example/docker-compose/scalable-single-binary/docker-compose.yaml

DONE: vault b server failed on initial run
DONE: 2/3 vaults sealed on startup, wat?

2024-03-09

DONE: run pg_upgrade in start of postgres, not recommended by patroni
DONE: on post_exit hook make sure we do patroni switchover
DONE: tls cert renewal vault
DONE: tls cert renewal consul
DONE: tls cert renewal nomad
TODO: add provisioning checksum to determine if should provision

2024-03-16

TODO: add tempo column to backend applications
TODO: provide region default tempo everywhere
DONE: add tempo integration tests
DONE: add all tempo clusters to grafana
DONE: add tempo docker images
TODO: for vault unseal only pass keys that are needed, no root token
TODO: grafana errors when logging in
TODO: backgrond spawns or tasks in EPL backend apps
DONE: enforce minio no bucket clash
DONE: instrument endpoints
DONE: instrument db queries
DONE: instrument NATS interactions

DONE: one span at the root
TODO: events for the rest of da things
DONE: maybe investigate having multiple spans for otel with same trace id?
TODO: app config

2024-03-17

DONE: too many metrics, we care only about apps, why the rest are traced? don't use tracing, use low level otel to specify what we want.

TODO: otel record errors, good for now
DONE: fix grafana errors with tempo
DONE: write new errors tests which were added with tempo
DONE: postgresql now doesn't do rolling update

Next: rolling updates, by default wait until healthy in nomad, but in MinIO case restart all at once
Added postgresql optional synchronous replication

2024-03-18

Rolling updates, then s3

TODO: add grafana for traces for all apps

2024-03-19

DONE: reduce minio min server count
They recommend 4 for production, but we use MinIO now mainly for docker images which is not intensive operations
DONE: add wirings for s3 buckets
DONE: add tests for all errors of s3 buckets wiring
DONE: codegen for backend apps
DONE: generate jobs declarations to use s3 buckets
DONE: test in single-dc environment s3 bucket, add manual integration test
DONE: test no bucket clash between apps and sys components, but multiple apps can use the same bucket
DONE: ensure you can't deploy app with minio cross region access

2024-03-20

DONE: backend app config support
DONE: support binary http bodies

2024-03-22

DONE: test raw http endpoints and edge cases
DONE: support streaming input body, which was async nightmare when I tried now
DONE: test no cross region s3 bucket usage

TODO: I want to handle full file upload
TODO: I want to query database
TODO: I want to perform db transaction
TODO: I want to use s3 bucket
TODO: I want to put stuff in queue
TODO: I want to serve json
TODO: I want to accept json input in http
TODO: I want to view traces
TODO: I want to view grafana dashboard
TODO: I want to view logs
TODO: I want to view vault secrets
TODO: I want to login to minio
TODO: I want to login to nomad
TODO: I want to login to vault
TODO: I want to login to consul
TODO: I want to view admin panel
TODO: I want to run something in the background
TODO: I want to run send configuration to my app

2024-03-24

DONE: implement backend app config validation
DONE: implement backend app config rust codegen
DONE: implement backend app config environment variable propogation
DONE: write backend app config integration test
DONE: write tests for backend app configs with validations
DONE: minio credentials provision issue, added stronger healthcheck to list buckets
DONE: disable defaults for postgres version to make sure user always specifies
TODO: check if minio admin password goes through http?
DONE: add pg_upgrade in the beginning if directory exists, all pg versions are pinned by default
pg upgrade is quite involved we need to do procedure of doing all needed steps in order
1. pause patroni
2. stop master
3. pg_upgrade with hard links
4. delete dcs state
5. restart patroni
https://patroni.readthedocs.io/en/latest/existing_data.html#major-upgrade-of-postgresql-version

2024-03-24

Coprocessor DC:
1. every coprocessor node has two wireguard connections p2p to two DC nodes
2. every connection node must have VPN and ospf
3. in multi dc mode we must connect to two datacenters
4. to exchange routing info, sending route every coprocessor node must be neighbor to the OSPF node

RULES:
1. There must be only one coprocessor datacenter in region
2. We mark VPN server as coprocessor_connection: true to set which servers coprocessors will connect to
3. There must be only two servers as coprocessor gateways
4. Coprocessor server must have two wg0 and wg1 interfaces

Coprocessor servers all form OSPF neighbor relationship with the nodes
Coprocessor servers attach virtual ip interface to the eth0 interface

TODO: data model
TODO: config generation
TODO: test

2024-04-02

DONE: write tests for the coprocessor errors

2024-04-03

DONE: coprocessor gw datacenter cannot have coprocessor gateways
DONE: test 4 more errors
DONE: clash single dc check with coprocessor dc check
TODO: move check for uniq constraint of network/interface to EPL instead of UNIQ because in coprocessor dc this can be true

2024-04-04

I had this before but I don't remember when or why
```
[admin@server-c.dc1.us-west.single-dc:~]$ ping -I 172.21.7.10 172.21.8.10
PING 172.21.8.10 (172.21.8.10) from 172.21.7.10 : 56(84) bytes of data.
From 172.21.7.10 icmp_seq=1 Destination Host Unreachable
ping: sendmsg: Destination address required
```

FIREWALL!! Fixed.

DONE: ospf routing to the coprocessor nodes

2024-04-06

TODO: freeze coprocessor environment
TODO: add virtual interface for coprocessor envs
TODO: move all LAN interfaces to bridges?
DONE: ospf different weights
DONE: single network for each coproc dc
TODO: subinterfaces tests
DONE: all coprocessor lan interfaces must have /32 prefix
TODO: restrict that you can only have multiple interfaces per network only in coprocessor datacenter
TODO: firewall for all traffic coming from internet to the public interface that tries to reach internal networks, nft rule that goes into interface and not to our ip drop all packets
TODO: :0 interfaces are not a thing now... think of what to do with secondary ip address
TODO: nomad consul interfaces don't specify ip, specify?
TODO: delete wireguard injected routes for nodes

2024-04-07

DONE: upping interface creates default routs, disable?

CLASH OF FN PREFIX ROUTE WITH INTERFACE UP AND DOWN
INTERFACE UP: ROUTES INSERTED
WIREGUARD RESTARTED: ROUTES RESTORED
HORRIBLE
OSPF WG INACTIVE ROUTE, NOT INSTALLED IF ROUTE NOT RESTARTED
MAYBE DEPENDS ON IP ROUTE?

FUCKING ROUTING MOTHERFUCKERS, THIS IS THE WORST PART OF THIS ENTIRE PROJECT

What if different wg subnet for servers?

2024-04-08

BGP list:
DONE: remove ospf neighbors
DONE: ospf auth is per single datacenter
DONE: use BGP pairing instead of OSPF
DONE: aggressive OSPF convergence time, 0.5 seconds hello, two seconds dead?

DONE: how to peer remote coprocessor gateways? we need BGP failover!! same remote as?
DONE: coprocessor configs

`OPEN Message Error/Bad Peer AS`

Can't establish BGP sessions, idle, study notification
Okay, ASN numbers were wrong, fixed
DONE: add bgp auth
DONE: add BFG

2024-04-11

DONE: refactor simulation freezing tests to make them easy to copy paste

2024-04-15

Project fast l1 prov

TODO: generate libsodium private key to sign the provisionings with
TODO: write OCaml daemon to check signature and load the script to tmux
TODO: Write systemd service which runs the daemon and watches consul kv yielding control to daemon
TODO: Write the rust routine to gzip, encrypt to target and sign the package by private key
TODO: Edit the makefile to upload the gzip to every region, extract and run the contents

2024-04-16

TODO: Refactor codegen to support binary files
TODO: use zstd instead of gzip
TODO: make sure provisioning id must be higher than existing not to play back old payload


2024-04-17
Compress

```
find . -type f | grep -v _build |\
  grep -E '/(default.nix|main.ml|dune-project|dune)$' |\
  tar -cvf l1-sig-checker.tar -T -
gzip -9 -f l1-sig-checker.tar
```

Untar
```
cat ../l1-sig-checker.tar.gz | gunzip | tar x -C .
```

TODO: now we need to generate l1 provisioning id and setup the consul service to listen for key
```
[root@server-a.dc1.us-west.single-dc:/etc/nixos/l1-checker]# ./result/checker /run/keys/l1-fast-prov-decryption-key /run/keys/l1-fast-prov-admin-pub-key ../plan.bin ../decr-plan.sh
Swap:             0B          0B          0B
    inet 10.17.0.10/24 scope global eth0
Parse error near line 1: no such column: L1_EPL_PROVISIONING_ID
  RT INTO l1_provisionings(provisioning_id) VALUES (L1_EPL_PROVISIONING_ID);
                                      error here ---^```

2024-04-18

DONE: can't locally evaluate stuff default.nix? add mock?
DONE: make sure we don't evaluate already seen l1 provisionings from consul with sqlite database
DONE: add check in l1 provisioning itself that if max l1 provisioning id is greater exit immediately
DONE: create script for doing all consul kv puts in the region with tmux
DONE: create script to compress all of the plans and send them via ssh to designated server
DONE: when ordinary l1 is finished insert into local sqlite that server is already bootstrapped
DONE: file exporter l1 provisioning id

2024-04-21

We can select non existant servers from sqlite like that.
We can generate cte... Do we need server list in txt file?
We refresh database every time?
Inside Makefile?
```
echo "with servers(hostname) AS ( values ('server-a'), ('server-b'), ('server-z') ) select hostname from servers where hostname not in (select hostname from bootstrapped_servers)" | sqlite3 infra-state.sqlite
```

.PHONY: refresh-server-infra-state
refresh-server-infra-state:
        echo " \
          DELETE FROM servers; \
          INSERT INTO servers(hostname) \
          VALUES \
            ('server-a'), \
            ('server-b'), \
            ('server-c'), \
            ('server-d'); \
        " | sqlite3 infra-state.sqlite


CI provisioning:
1. cat l1 fast provisioning to get l1 provisioning id
2. query servers that are not yet bootstrapped and do those
3. upload the rest of plans, idempotent if already done

TODO: ci provisioning, first provision servers with internet?

TODO: add alert for diverging l1 provisioning ids

2024-04-22

TODO: edendb regression
should return normal error that we have double tables defined but assert is triggered
```
#[test]
fn test_double_foreign_child() {
    assert_compiles_data(
        r#"
TABLE server {
  hostname TEXT PRIMARY KEY,
  is_consul_master BOOL DEFAULT false,
  is_nomad_master BOOL DEFAULT false,
  is_vault_instance BOOL DEFAULT false,
  is_dns_master BOOL DEFAULT false,
  is_dns_slave BOOL DEFAULT false,
  is_ingress BOOL DEFAULT false,
  is_vpn_gateway BOOL DEFAULT false,
  is_coprocessor_gateway BOOL DEFAULT false,
  is_router BOOL DEFAULT false,
  kind TEXT DEFAULT 'dc_default',
  run_unassigned_workloads BOOL DEFAULT true,
}

TABLE server_volume {
  volume_name TEXT PRIMARY KEY CHILD OF server,
  mountpoint TEXT,
  source TEXT,
  UNIQUE(hostname, mountpoint),
  UNIQUE(hostname, volume_name),
}

TABLE ch_keeper_deployment {
    deployment_name TEXT PRIMARY KEY,
    loki_cluster TEXT DEFAULT region_default,
    monitoring_cluster TEXT DEFAULT region_default,
    workload_architecture TEXT DEFAULT x86_64,

    keeper_port INT DEFAULT 9181,
    raft_port INT GENERATED AS { keeper_port + 1 },
}

TABLE ch_keeper_deployment_instance {
    instance_id INT PRIMARY KEY CHILD OF ch_keeper_deployment,
    keeper_server REF FOREIGN CHILD server_volume,
    keeper_server REF FOREIGN CHILD server_volume,
    UNIQUE(keeper_server, deployment_name),
    UNIQUE(instance_id, deployment_name),

    CHECK { instance_id > 0 },
}

"#,
        json!({
        }),
    );
}
```

DONE: add check that keeper servers are 3 or 5 for quorum
DONE: fix prometheus metrics scraping for clickhouse keeper
DONE: fix keeper logs
DONE: write keeper fail tests
TODO: add job name to all metrics integration tests? not all clusters may contain specific cluster metrics


* 2024-04-24

DONE: clickhouse interserver credentials with vault
<!--<interserver_http_credentials>
    <user>interserver</user>
    <password></password>
</interserver_http_credentials>-->

DONE: add clickhouse integration tests
DONE: add clickhouse credentials for all users

TODO:
Warnings:
 * Delay accounting is not enabled, OSIOWaitMicroseconds will not be gathered. You can enable it using `echo 1 > /proc/sys/kernel/task_delayacct` or by using sysctl.

* 2024-04-25

DONE: precise memory requirements for clickhouse with config

* 2024-04-27

DONE: generate interserver clickhouse credentials
DONE: generate master default password
DONE: admin utility to connect to local instance shell
TODO: clickhouse keeper ACL
ACLs are so complex in zookeeper and ch keeper and undocumented, and barely provide any business value so we might as well just skip them for now.
DONE: add nixpkgs tarballs checksum for verification
DONE: move /clickhouse/sessions path

Test snippet to create table
```
CREATE TABLE table_name ON CLUSTER default (
    x UInt32
) ENGINE = ReplicatedReplacingMergeTree
ORDER BY x;
```

TODO: clickhouse schemas for users
TODO: nats to ch table

Clickhouse - prefer HTTP driver because it is stable and well documented

* 2024-04-30

DONE: clickhouse implement batch execute
DONE: parse split and statically analyze ch queries
DONE: add error for adding Replicated* tables, these are created underneath
DONE: add engine checks only MergeTree engines are allowed
DONE: enforce that every CREATE TABLE statement is followed with IF NOT EXISTS
DONE: enforce that every CREATE VIEW statement is followed with IF NOT EXISTS
DONE: enforce that every CREATE MATERIALIZED VIEW statement is followed with IF NOT EXISTS
DONE: enforce that every DROP TABLE statement is followed with IF EXISTS
DONE: enforce that every DROP VIEW statement is followed with IF EXISTS

* 2024-05-02

TODO: figure out json inserts for clickhouse, the current way from video is to query json itself?
DONE: test clickhouse engine
DONE: write schema tests

* 2024-05-03

TODO: last empty downgrade is not tested
DONE: convert query parameters for ch to use {wookie:Int32} parse_full_query is specific to Postgres

* 2024-05-05

DONE: create migration user for Every DB
DONE: create rw user for Every DB
DONE: create ro user for Every DB
DONE: Upgrade clickhouse
DONE: perform all migrations through db admin user

* 2024-05-07

DONE: create migrations table
DONE: replace da thing
DONE: clickhouse quorum tests
DONE: prevent backticks in clickhouse DDL test
DONE: add non snake case for column test for clickhouse
DONE: add non snake case for column test for postgres
DONE: make sure in clickhouse we test only on single database with appropriate permissions to test all migrations with, no CLUSTER grant
DONE: add check that there must not be `ON CLUSTER` inside schema ddl

* 2024-05-07

DONE: migrate SQL params to clickhouse convention
DONE: migrate URL params to {} instead of <>
DONE: add non snake case for table test for clickhouse
DONE: add non snake case for table test for postgres
DONE: only after async checks we know that tables exist in ch schema, check then if all inserters are valid
DONE: factor out types from query parsing, clickhouse types should be separate from pg
DONE: implement wiring checking for application ch shards

* 2024-05-08

DONE: nomad codegen for configuring clickhouse db shards
DONE: add check that backend application deployment is in same region as ch instance

* 2024-05-10

DONE: add ch shard wiring tests
DONE: rust codegen for clickhouse dbs
TODO: clickhouse no TLS plaintext password inside subnet

* 2024-05-11

ch rust codegen:
DONE: struct variables
DONE: env variables
DONE: add base64, unescaper crate
DONE: reqwest client init
DONE: query struct fields
DONE: inserter fields with optional values
DONE: api function for selector
DONE: ChInteracionError
DONE: Clickhouse forbid nullable columns
DONE: if default is provided make field optional
DONE: write rest of the fields as is
DONE: fix ch query so that we could receive which columns have default values
DONE: api function for inserter

* 2024-05-12

DONE: test the inserter, add manual integration test
DONE: shell.nix for the apps so its easy to develop
added .envrc 'use flake', codium . to develop
TODO: Support opt fields for clickhouse queries, now it is technically supported but we don't support any Nullable type from a query

* 2024-05-14

DONE: NATS to clickhouse table
DONE: check that clickhouse deployment and nats cluster are in the same region
DONE: check that schemas of clickhouse and bw compatible type match
DONE: write tests for stream import regionality
DONE: write test for trying to insert column which is materialized or alias
DONE: write tests for schema mismatch
DONE: nats queues json instead of binary for clickhouse? or maybe...
DONE: generate clickhouse consumer codegen
DONE: test with integration test
TODO: queries verified multiple times fail multiple insertion test

TODO: NATS stream provision issue
```
030 provision-nats-resources.sh     | nats: error: could not create Stream: no suitable peers for placement (10005)
```

* 2024-05-15

DONE: schema for blackbox
DONE: add detached default for region
DONE: test_into_table_column_type_unsupported
TODO: placement labels for stateless blackbox deployment

* 2024-05-16

DONE: test group count for stateless workloads must be 1
DONE: add memory to tasks
DONE: stateless port lock
DONE: add configs
DONE: add entrypoint parsing
DONE: add arguments parsing
DONE: create a job with templates and secrets

* 2024-05-19

DONE: placements
DONE: add integration tests for services with instances
DONE: server volume mounts

blackbox deployments secrets subproject:
TODO: add static analysis in templates if we're getting right secret
TODO: support vault secrets
TODO: add secrets as env variables
TODO: add secrets as files
TODO: add static analysis in templates if we're getting right secret
TODO: force check that min_instances in service registration is enforced

TODO: add monitoring that service exposes only allowed ports in its range

TODO: not all nomad jobs scheduled? Do retries and try to see if job is okay?

TODO: rootless docker?

* 2024-05-21

TODO: nomaster on postgres, so some replicas can never be masters

acme validation
DONE: /var/lib/acme encrypted zfs dataset for storing state
TODO: vault policy + token for updating certificate + private key
TODO: make nginx use the certificate from vault if it exists
DONE: generate rfc2136key
DONE: add acme config to have renewed certificates
DONE: FN bind can't update zone without public ip!!! use server public ip for update? That will be legit...

DONE: save epl cert in vault
/var/lib/acme/epl-infra.net/cert.pem
/var/lib/acme/epl-infra.net/chain.pem
/var/lib/acme/epl-infra.net/fullchain.pem - this is good for nginx
/var/lib/acme/epl-infra.net/key.pem - this is private key

* 2024-05-23

DONE: generate vault policy, should this be in vault? we could init policy but don't create a token. We need to provision this as part of l1 secret, how nomad does this?
DONE: generate post success secret to vault update
DONE: distribute acme vault secrets to all regions
DONE: separate kv not by prefix but by slashes
DONE: make sure external lb uses the certs

* 2024-05-24

DONE: secret is not gotten from vault, fix! Empty when generating secret? It should get existing one.
DONE: add tmp secret declaration from custom scripts from DNS
TODO: don't fail service if cert loading failed, its repeated daily
TODO: rename to statement in clickhouse renames zk path, which sucks, how do we migrate now? we say we don't support, add test for not supporting table renames
TODO: push tars directly to docker registry

* 2024-05-28

DONE: embed schema inside compiled epl executable, do something with release?

* 2024-05-29

DONE: tidy executable build to epl
DONE: nginx hot reload on cert change

DONE: clickhouse tests tuning for retries
thread 'tests::clickhouse::queries::test_query_unused_argument' panicked at src/static_analysis/databases/clickhouse.rs:1821:25:
Failed to start clickhouse database in 20 seconds: invalid params: empty host

TODO: prepare epl shell as a product

TODO: move metrics db from edb-src to the project, and have metrics for every cluster instead

redo the alert tests algorithm:
1. test all the alerts with their alert groups
2. if alerts weren't tested in first specific phase then join all the clusters alerts together and test with those

* 2024-06-01

DONE: alert prom metrics db
DONE: test metrics refresh workflow
DONE: fix alerts tests
TODO: distinguish dev environment vs real environment in makefile
TODO: write basic metrics for running out of disk space for zpool and stuff
DONE: make sure most syntax checks are processed if db is not found?
DONE: pick epl executable that is is from dev env if exists or just from path automatically
TODO: move out library.sh to root
TODO: figure out how to move shell.nix to the compiler project for independence?
there's a problem with `third-party` patched nix generators...
TODO: auto generate nix hardware configs for bare metals

DONE: test nats stream provision
TODO: MinIO start sucks, how to make it better?


* 2024-06-02

DONE: simple bare metal DC implementation
DONE: passive vpn for datacenter, where connection is not IP to IP but only one side connects to public ip

* 2024-06-04

DONE: instead of panic normal error for sm simple more than one subnet
DONE: test more than one subnet in bm simple
DONE: add codegen for gateway ip
DONE: check gw ip belongs to subnet
DONE: fix all tests

* 2024-06-09

DONE: rename tank to rpool root zpool

* 2024-06-13

DONE: add disk ids to server disks, optional column that is always preferred if exists
DONE: check that if we add disk id to one column we must add to all
DONE: check that disk serial is defined it must be unique across all disks
DONE: precondition for disk by id
DONE: write test for disk checking
DONE: precondition check that serial resolves to disk id

```
[admin@nixos:~]$ ls -lha /dev/disk/by-id/
total 0
drwxr-xr-x 2 root root 540 Jun 15 12:42 .
drwxr-xr-x 9 root root 180 Jun 15 12:42 ..
lrwxrwxrwx 1 root root   9 Jun 15 12:42 virtio-server-a-vdb -> ../../vdb
lrwxrwxrwx 1 root root   9 Jun 15 12:42 virtio-server-a-vdc -> ../../vdc
```

* 2024-06-15

DONE: regionality distribution test function
DONE: implement regionality distribution function inside the domains
DONE: add tests for regionality distribution function

* 2024-06-16

TODO: should 4 ingresses be allowed if one DC doesn't have public ip?
TODO: IPv6 support for internet

1. add ipv6 public column on ingress, if its not empty verify valid public ipv6
2. add dns records for ingress
3. modify nginx config to listen on ipv6 for requests
profit?

* 2024-06-23

TODO: metrics collection for compile time. do I need tests for that? It just needs to be valid prometheus expression that exists? test with getting minimum value from db? But wait, if prom expression is wrong we'll just find out during refreshes and we can log anyway?

```
curl http://10.17.0.10:9090/api/v1/query\?query\=postgres_exporter_build_info
```

* 2024-06-27

TODO: disk drive letters inconsistent across reboots, how to refactor now? universal disk id?

* 2024-06-28

DONE: vm image selects default os version, pin it!
DONE: single-dc-disk-serial fix root zfs disk id
TODO: single-dc run and test

```
Unexpected root disk id for root zpool of rpool, expected [virtio-server-a-vda] actual [vda]
```

* 2024-07-07

nats enable subjects:

DONE: if jetstream stream has subjects enabled, make sure app interacts through subjects also
DONE: on publishing format subject by adding to the signature
DONE: on subscription make sure we pass in subject to the api functio

* 2024-07-08

TODO: about disk partitioning, maybe we can check that partitions are not wiped for zpools and will fail creation if zpool doesn't exist yet? Or maybe just add -f to force it?
TODO: blackbox deployment prevent non ending with \n for the file

* 2024-07-09

DONE: fixed DNS setup for admin services
TODO: add dns root zone checks into preconditions
TODO: add dnssec for aws

root zone requirements:
1. delegate _acme-challenge with separate NS record
2. update SOA to match bind config_
3. update NS for root to match bind config
4. add A records for NS servers

* 2024-07-10

DONE: telegram alerts
TODO: move telegram bot token to secrets
TODO: add validations for telegram chat id
TODO: add validations for telegram api key

* 2024-07-11

DONE: custom makefile includes
TODO: regression: if in clickhouse table is created but wrong existing table is dropped then it is segfault
DONE: provisioning ww for one server provisioning id diverges, is recomputed


* 2024-07-14

TODO: monitor queued jobs 'nomad_nomad_job_summary_queued > 0'

TODO: try with password
```
➜  p1 git:(scylla_upgrade) ✗  echo -n cWNASkBvCF0kyOFFJSGOPmuir48An35pHh5NoBvC6v | mkpasswd -s
$y$j9T$jY/9Iu2aCx4VZD/0dRyUR0$D1vMRY6g7HJmMCblZqWkki/IVEqowKpwVcbcWL7VKMA
```

1. make sure telnet waits for first bootstrap node
2. make sure we salt the password for scylladb

* 2024-07-15

DONE: scylla dropped
DONE: refactor tests after column drops

* 2024-08-03

DONE: separate alertmanager from victoria metrics, these we can have 3-5 instances, but for prom/vm we need only two
TODO: unified CI provisioning
DONE: fix env tests
DONE: fix for defining am

FKING BIND!!! automatic journal edit?
```
journal rollforward failed: journal out of sync with zone
zone not loaded due to errors.
```
DONE: prevent inserters on views
DONE: log cleanup for provisionings

* 2024-08-15

DONE: ipv6 check field for every server if not empty if it is valid public ipv6 ip
DONE: extract ipv6 address from terraform with jq
DONE: ipv6 dns PTR reverse lookups
DONE: enable firewall rules for ipv6 DNS and http/https traffic
TODO: generate root zone glue records for all tlds
DONE: figure out ipv6 networking settings for aws for NixOS
TODO: alert for vault sealed nodes
DONE: DNS records for ipv6 master servers !!!
TODO: log retention for loki
DONE: nomad server hardly recovers after restart, think of mechanism how to auto restore it with zfs snapshot/delete? just create secret volume

1. ping if other servers exist, if there are two candidates, make zfs snapshot + rm all files + start?

* 2024-08-18

Nomad namespaces:
1. remove `epl-` prefix from nomad jobs/tasks/task groups
2. keep consul prefixes the same
3. refactor all `fetch_nomad_job` calls
4. inside codegen pick the namespace

TODO: designated l2 provisioning node for region
TODO: designated fast l1 provisioning node for region

* 2024-09-18

TODO: root cause of dns failure, cert update fails.
we cannot refresh our dnssec keys. they're never uploaded.
our dns stops working...

WE MUST SEPARATE LEGO ACME UPDATE BULLSHIT FROM MAIN NIX FLOW
Cert update failure should't stop l1 provisioning FFS

DONE: vault key to update dns credentials expired, make sure we run hourly job to renew the token

TODO: add monitoring on cert expiration, it should all be automatic, gtfo

* 2024-09-22

1. Refresh token script, boom
2. Detach acme certs from l1 provisioning
3. Update load balancer secret as well

DONE: make sure space before command doesn't keep in history

DONE: high level plan for the provisioning
1. build aws images, already based on file
2. if main.tf file is modified we need to reapply it, compilation will not modify file if its the same
3. have marker directory markers/public-node-bootstrap/server-a.marker, that we bootstrap public node
4. start VPN - instant, do this always
5. aws private ips bootstrap internet - do this always if there's no l1 provisioning marker for the server
6. l1-provision-with-wait - we have markers/l1-provisioning-done/server-a.marker that is touched always after success
7. consul bootstrap - markers/consul-bootstrap/<region>/server
8. metrics scraping to make decisions on current infra

FLOW:
- query all metrics
- compile the project
- build aws images
- run terraform
- run l1 fast for all nodes that support
- for the rest of the nodes run l1 with ssh
- run l2 provisioning

DONE: full-provision-pre-l1 adapt for lazy eval, bootstrap public servers
DONE: full-provision-pre-l2 adapt for lazy eval, bootstrap public servers
DONE: leftover with vault variables, now warning is emitted
some lua error that checks certain metric name can only have one value
TODO: create sqlite database with expected hashes and scrape from metrics, now we can skip l1 fast provisioning if matches
DONE: check if we can skip l1 completely if all server hash state matches?
TODO: one nix build for all apps with their dependencies
DONE: vault unseal when testing single server, wth?
it's dns... again
TODO: something that executes `nomad system gc` with token if there's client deregistration issue?

TODO: consul exporter to detect when postgresql has two master replicas in consul
TODO: container_start_time_seconds doesn't have nomad job name label
DONE: sqlite function to extract metrics
DONE: save l1 bootstrap hash inside sqlite
DONE: filter out servers already converged
DONE: what to do about server reboot? what about boot id? node_boot_time_seconds

this will get boottime
```
cat /proc/stat | grep btime | awk '{ print $2 }'
```

DONE: l1 provisioning record boottime inside the metrics, then we'll know if l1 provisioning matches and we need to reprovision with bootstrap!

DONE: reject if metrics scrape is older than 30 minutes
DONE: reboot alert doesn't work because node is disconnected from consul
TODO: can't do vault unseal if vault node has rebooted, need l1 first
TODO: if metric scraping fails check if VPN exsists and if servers aren't bootstrapped turn it on

TODO: alert if node is over given memory limit than `memory` but less than `memory_max`

* 2024-10-26

DONE: clickhouse mutators for ETL automation
DONE: implement resulting data for mutators for pg_mutator
DONE: check that ch mutator and query names don't clash
DONE: check that resulting data doesn't exist before query, if any row exists it is bad
DONE: after the query ensure that every single row in test dataset exists in output table
DONE: ensure we're only executing mutator with fresh test data inserted
DONE: check that test dataset at least one column is specified
DONE: write tests for every error of data checking
DONE: test for duplicate mutator and query
DONE: codegen for ch mutators
DONE: write tests for application mutators shards checking
DONE: write tests for postgresql resulting data all errors

TODO: docker image was not build right away when testing, how can we help?
TODO: clickhouse oomed in the start when testing, how can we help?
TODO: is it an issue that in postgresql empty row is specified when inserting test data?
TODO: integration test for ch mutators

* 2024-10-27

TODO: add i128, i256, DateTime tests for clickhouse
TODO: add LowCardinality(..) table tests to clickhouse
TODO: flake.nix add developer shell with dependencies
TODO: lock crane dependencies, or use naersk instead
TODO: Cargo.lock was not upgraded even if deps were added, how to automate?
- prevent l2 if all apps can't build?
TODO: Correct permissions for rust app deployment if mutators are used?
TODO: If image tags aren't replaced in nomad job don't deploy
TODO: rw doesn't work for clickhouse, wat?
TODO: app clickhouse secrets weren't refreshed by vault because of incorrect permissions, cmon
TODO: hyper::Error(IncompleteMessage) when doing clickhouse mutator
https://github.com/hyperium/hyper/issues/2136

TODO: what to do with docker image being pulled first time?
TODO: set tracing statuses on errors in codegen
TODO: revisit nats jetstream consumer tracing
TODO: alert on consul service logs "rejecting RPC conn from because rpc_max_conns_per_client exceeded"
TODO: clickhouse too many simultaneous queries
make this customizeable '<max_concurrent_queries>100</max_concurrent_queries>'

* 2024-11-11

TODO: vault unseal doesn't always work after restart
remedy: `systemctl restart vault.service` and unseal again
```
core.cluster-listener: no TLS config found for ALPN: ALPN=["raft_storage_v1"]
```

* 2024-11-13

DONE: check if implementation directory exists and don't write example file then
TODO: add all generated files to git if they're not binary
DONE: add victoria metrics retention, now it is 2 years hardcoded

* 2025-01-09

DONE: vlan in hetzner should just use same internet interface, with `_vlan` prefix?
DONE: check that network interface ends with vlan id
DONE: check that interface with vlan is attached to another interface


zpool host id issue fix
```
zpool set multihost=on rpool
zpool set multihost=off rpool
zpool status rpool
```
DONE: fix vlan issue for nomad startup

Draining nomad masters procedure:
- reconfigure different masters
- apply l1 provisioning
- call 'nomad system gc' with bootstrap token`to fix the state

TODO: acme certs interfere with first time dns master setup, chicken and egg problem, now we need manual disabling of auto certs
DONE: make sure you pick one address from IpV6 prefix during compile time

TODO: external load balancer running only one instance, grafana forwarding bullshat behind VPN?

DONE: compute projection of available blackbox deployment resources
DONE: compile time errors and static analysis checks for blackbox deployment resource
DONE: actual codegen to deliver the secrets to the blackbox deployment
TODO: postgres default read only user and password
DONE: write tests for new errors regarding blackbox deployment variables

TODO: NixOS upgrade 24.11
TODO: Add monitoring for expected job group counts/running job group couns
TODO: How to deploy, push source ir build locally with nix?
TODO: figure out location paths and stuff on the morrow
DONE: DNS for load balancer names
TODO: bind reload didn't work for the zones, oopsie, findout by tweaking bind records
TODO: patroni new version error https://github.com/prometheus-community/postgres_exporter/issues/1060
TODO: fix min instances for blackbox service bug, we have enough instances but it is not registered that that many servers have it
TODO: universal provision doesn't include server that exposes the hashes
TODO: bind isn't restarted when adding new certificate tld?
DONE: dns sec enabled but unsupported by upstream TLD
TODO: make dnssec root zone validation as part of universal provisioning
TODO: bound stateful job with stateless task has no server locks error
TODO: Arbitrary DNS records for domains
