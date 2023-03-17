# pgext

## Usage

```
cargo run -- init ~/.pgx/15.2/pgx-install/bin/pg_config ~/.pgx/data-15/ ~/.pgx/15.2/contrib/
cargo run -- install pgjwt
```

Currently we only support tar/zip-packed PGXS extensions.

## Hook Detect Extension

This extensions will show all function address of hooks in the Postgres. To build and use it,

```
cargo install cargo-pgx@0.7.4 --locked
cargo run -- install-hook
```

Then run SQL:

```
SELECT * FROM show_hooks.all();
```

## Test Hooks

```
cargo run -- test pg_stat_statements
cargo run -- test-all --dump-to result.csv
```
