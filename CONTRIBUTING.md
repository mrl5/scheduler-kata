# Contributing

Install dev tools

```console
cargo install just
just dev-tools
```

## HOWTO bootstrap database

### One time bootstrap

You will need two terminals:
1. Let's create our container with vanilla postgres first
```console
just db-only
```

2. We have vanilla postgres with empty database for the project. Now let's
   bootstrap `pg_cron`. Run it in 2nd terminal:
```console
just db-load-pgcron
```

3. Container was restarted. You can attach to it in 1st terminal again:
```console
just db-only
```

4. Let's continue the bootstrap process in 2nd terminal:
```console
just db-bootstrap
```

### Per tenant migration

Each time you provide new `TENANT` (e.g. in `.env`) you will need to run:
```console
just db-add-new-tenant db-migrate
```

For more info inspect the content of `.env` files. Notice the value of `TENANT`
variable.


## HOWTO local dev

Unit tests
```console
just db-only
just --dotenv-filename .env.local test-unit
just --dotenv-filename .env.local local-api
just test-api
```

Before `git commit` you can run
```console
just lint
```
