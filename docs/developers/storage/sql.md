# SQL Storage Backend Support

> Not (yet) recommended for production use

Champ allows node operators to choose a variety of storage backends, one category of which is based on rational databases. Supported are SQLite, Postgres, and MySQL. This is provided as an option to improve speed and fault tolerance, so SQLite is not recommended, and operators should use our sled-based storage backend for these simpler, self-contained deployments.

## Entity Relationship Diagram

![](./entity.drawio.svg)

## A cryptocurrency running on a SQL Database?

While it might - from a first glance - seem like this enables node operators to forge and edit transactions easily, this is the case for any cryptocurrency. The whole point of cryptocurrencies is to have verifiability independent of a single actor. While individual node operators can operate in bad faith, the greater network can detect this, reprimand individual nodes, and verify any transaction independently. While you can change different fields, you can't (at least without a currently impossible quantum computer) forge signatures verifying each action.

Most cryptocurrencies are built on simple key-value stores like `rocksdb` or `lmdb`. While our primary/default storage backend is also based on `sled`, which is in the same category as these, we decided to build out a SQL-based alternative. This is mainly to enable some exciting use-cases for analysis purposes on the live network and to see if our assumption that a simple storage backend doesn't necceceraly improve speed and reliability compared to a battle-tested hyper-optimized database as PostgreSQL holds.

## Our ORM

We've decided against using raw SQL for various reasons, like development speed. However, first and foremost, we wanted compatibility with a variety of databased backends without the need to write different dialects of SQL ourselves. We've ended up with [SeaORM](https://github.com/SeaQL/sea-orm), mainly because of its great async support and our great experience talking to their developers.

## Migrations

We've already set up migrations; however, these will only start being created once we reach our first "stable" version (Around Summer 2022). Until then, we're utilizing our ORM's schema generation feature.
