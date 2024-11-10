# Contributors Guide

Forty years long was I grieved with this generation, and said, It is a people that do err in their heart, and they have not known my ways: - Psalms 95:10

One day I will leave this earth and forever be with the LORD. Needless to say, I'll have better things to do then than developing Eden platform. If you want to become contributor to this project you need to adopt this mindset to how this project was made. If you don't like something, I don't care, but you have option to fork off (permissive BSD license) and do things in your own (likely inferior) way.

## Think if feature needs to be added

Before you think of adding any new feature first think and research if something you need to do can already be done with what Eden platform already supports.

I'll give you an example. You might look into Eden platform and see that it only supports Postgres now. I'll add MySQL! No, you will not. All users are expected to use Postgres to avoid confusion. A lot of features of MySQL and Postgres overlap. We want to avoid maintaining two databases and incompatibilities between them. For instance, we can have logical replication across multiple tables with Postgres. With MySQL we cannot. If we were to support MySQL then if we specify that we want logical replication of a table into another database we'll have issues when database is not MySQL, we'll need more compiler errors to cover and etc. But, if we assume we only use Postgres everywhere we can make more assumptions and take advantage of all the features PostgreSQL can offer.

For every feature you'd like to develop first see what is already available and if your problem can be solved with that. Don't add more components which offer 90% the same functionality from different component. Don't think about adding RabbitMQ, or Kafka next to NATS because you're in a mood swings today (are you a woman?).

## Default is a compile time error

When adding features and compiler errors, compiler error is the default. You need to disable checks explicitly.

Example is, we check that no query does seq scans in a database. This is the default and will always error out. This check can be disabled if you know that table is small by specifying seqscan_ok to true in db_query column. This should never be the other way around, that check is disabled by default and user needs to add this, it is easy to forget. Always default to compiler error, if error is not critical and can be turned off, ensure user needs to specify that check is disabled for this context explicitly.

## Don't allow users to make mistakes

If some input from user is yaml, the most horrible data format of all time, we always parse yaml strictly. If any field in yaml is extraneous, we don't silently ignore it (annotate struct with `#[serde(deny_unknown_fields)]`). Error out for any extra field that is not used by us. That's how we can turn yaml into bearable data format. We parse it instantly, and analyze data instantly if it is correct and valid with the rest of the system, then we can show user errors right away instead of doing that in production like most yaml faggots do today and blame users for having bad data.

## Think of as many cases as possible of how user can mess up inputs

Someone I know said "don't do defensive programming". That person is highly respected, does talks, workshops, almost sounds smart when he's talking and etc. But that person is an imbecile who never wrote a compiler and only ever wrote shitty backend apps and microservices that breakdown due type errors in production.

We catch as many user mistakes as we can before shipping to production. That means, there are a lot of return statements for errors in Eden platform code. Find an excuse not to compile users code, then get out as soon as possible with an error.

## Be responsible

If you lack some sort of information to make a decision, just take ownership, get that information and make assumptions.

For instance, there is this mental disease in the world, if idiot was trying to implement Eden platform and he decided to use third party DNS services, he'd stumble upon the dreaded question:
- We don't know what DNS service provider user will use, hence we must be generic about it!
Instead, what I did in the Eden platform is run your own DNS. Yes, it took effort to implement Bind9, but now we control how it works and can make our own assumptions. The end user just needs few records to point to our authoritative DNS servers and the rest of DNS is ours. And these same bootstrap DNS records are generated with DNS control, but if DNS control doesn't support certain provider user can simply set initial DNS records himself and forget it.

We didn't throw hands into the air like a faggot "we don't know what DNS provider user will use!" but we own our own DNS and now we can make assumptions about how it works and avoid endless bad generic decisions made due to lack of information about the end user system.

## Avoid generic interfaces

For instance, there three types of tables for interacting with db at this time:
- db_query
- db_mutator
- db_transaction

We assume about characteristics of these queries.
- db_query returns vector of structs
- db_mutator must mutate something in the table
- db_transaction takes multiple queries and mutators together and generate step execution structs for performing a transaction.

Some faggot might say "db_query shouldn't return a vector, but a stream, what if its bazillion rows and it can't fit into memory?". Well 99% of the time queries return a few rows.

Now, that doesn't mean we shouldn't support streaming queries, so far it's not implemented, but if it will it will probably be implemented as `streaming` boolean column which defaults to false in `db_query` table and now that query will return future of all the rows from that query.

A faggot would make only one db_query table which does queries, mutations, transactions and user would get a generic, good for no specific case interface. Multiple specific abstractions are better than one generic abominable abstraction about which assumptions can't be made.

Which leads into the next point...

## Bind to components and make assumptions

We use Postgres for a database (did I mention this yet?). That will never change and this flavor of Eden platform will never support any other relational database. So instead of having problems, like developing generic JDBC driver to support all databases in the world, we can and should simply make assumptions about what database we use. This paves the way for implementing Citus for horizontal scaling, paves way for using all advanced Postgres features like logical replication, full text search, pg_notify and etc. We don't have to think "but this database feature is non-standard and will not work on every database" and avoid all the issues about standards compliance and whatever. We fully bind to the components that we use and fully take advantage of them in the Eden platform. Also, we avoid bleeding edge, weekend faggot projects with nice looking logos that didn't stand a test of time and whose future is not clear to ensure things are stable.

## High level picture is all that matters

Low level details are irrelevant. For instance, for provisioning resources we now run shell scripts. They're dynamically typed, sometimes look abominable but we generate them so it is part of our low level, irrelevant swappable assembly that we don't touch by hand. In future this might be changed, but who cares? Is it an issue? In codegen stage of our shell scripts entire database has been checked and we assume we have a working state. So, we assume that our code that we generate is correct. We have integration tests for test projects to make sure database was provisioned, NATS stream exists and etc (which tests user doesn't need to have as it should just work for him). Shell scripts are a low level irrelavant detail of how something is provisioned and user deals with high level picture where all mistakes will be caught in his infrastructure before shipping.

Everything that is beneath the highest level goal of helping user ship software without mistakes and run his rock solid infrastructure with blazing fast performance can and should be changed. Infrastructure should serve the user, the user shouldn't serve the infrastructure.
