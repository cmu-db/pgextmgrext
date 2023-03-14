# pgext

## Usage

```
cargo run -- install https://github.com/michelp/pgjwt/archive/9742dab1b2f297ad3811120db7b21451bca2d3c9.tar.gz ~/.pgx/15.2/pgx-install/bin/pg_config
```

Currently we only support tar-packed PGXS extensions.

## Tested Plugins

* https://github.com/michelp/pgjwt/archive/9742dab1b2f297ad3811120db7b21451bca2d3c9.tar.gz
* https://github.com/ossc-db/pg_hint_plan/archive/refs/tags/REL15_1_5_0.zip

## Hook Detect Extension

This extensions will show all function address of hooks in the Postgres. To build and use it,

```
cargo install cargo-pgx@0.7.3 --locked
cd pgx_show_hooks
cargo pgx run pg15
```

Then run SQL:

```

CREATE EXTENSION pgx_show_hooks;
SELECT * FROM show_hooks.all();
DROP EXTENSION pgx_show_hooks;
```
