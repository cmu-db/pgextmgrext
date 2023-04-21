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

## Pgext Framework Quickstart

```
# Install everything
cargo run -- install-hook
# Compile two plugins
cd test_plugin && make PG_CONFIG=~/.pgx/15.2/pgx-install/bin/pg_config PG_LDFLAGS=-Wl,-U,___pgext_before_init,-U,___pgext_after_init install
git clone https://github.com/skyzh/pg_hint_plan/ && cd pg_hint_plan && make PG_CONFIG=~/.pgx/15.2/pgx-install/bin/pg_config PG_LDFLAGS=-Wl,-U,___pgext_before_init,-U,___pgext_after_init install
git clone https://github.com/yliang412/pg_stat_statements && cd pg_stat_statements && make USE_PGXS=1 PG_CONFIG=~/.pgx/15.2/pgx-install/bin/pg_config PG_LDFLAGS=-Wl,-U,___pgext_before_init,-U,___pgext_after_init install

# Modify the config to include all three extensions
cargo run -- test pgext_framework pgext_test_plugin pgext_pg_stat_statements pgext_pg_hint_plan
```
