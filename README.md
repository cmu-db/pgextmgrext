# pgext

## Usage

```
cargo run -- init ~/.pgrx/15.2/pgrx-install/bin/pg_config ~/.pgrx/data-15/ ~/.pgrx/15.2/contrib/
cargo run -- install pgjwt
```

Currently we only support tar/zip-packed PGXS extensions.

## Hook Detect Extension

This extensions will show all function address of hooks in the Postgres. To build and use it,

```
cargo install cargo-pgrx@0.8 --locked
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
cd test_plugin && make PG_CONFIG=~/.pgrx/15.2/pgrx-install/bin/pg_config PG_LDFLAGS=-Wl,-U,___pgext_before_init,-U,___pgext_after_init install
git clone https://github.com/skyzh/pg_hint_plan/ && cd pg_hint_plan && make PG_CONFIG=~/.pgrx/15.2/pgrx-install/bin/pg_config PG_LDFLAGS=-Wl,-U,___pgext_before_init,-U,___pgext_after_init install
git clone https://github.com/yliang412/pg_stat_statements && cd pg_stat_statements && make USE_PGXS=1 PG_CONFIG=~/.pgrx/15.2/pgrx-install/bin/pg_config PG_LDFLAGS=-Wl,-U,___pgext_before_init,-U,___pgext_after_init install

# Modify the config to include all three extensions
cargo run -- test pgextmgr pgext_test_plugin pgext_pg_stat_statements pgext_pg_hint_plan
```
_
# Lints

```
cargo +nightly fmt
cargo clippy
```