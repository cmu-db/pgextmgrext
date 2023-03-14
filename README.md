# pgext

## Usage

```
export PG_CONFIG="~/.pgx/15.2/pgx-install/bin/pg_config"
cargo run -- install pgjwt
```

Currently we only support tar/zip-packed PGXS extensions.

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
