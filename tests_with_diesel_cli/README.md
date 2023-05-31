The purpose of these tests are to make sure the this crate integrates nicely with `diesel_cli`. First we run with default configuration (created by `diesel setup`), then with a custom config `custom.diesel.toml` which instructs `diesel_cli` not to generate the mapping types.

To run these tests locally:

* Make sure you have `libpq` installed
  - You may have to mess around with your paths etc, see [stackoverflow](https://stackoverflow.com/questions/44654216/correct-way-to-install-psql-without-full-postgres-on-macos)
* Also install `docker`/`docker-compose` and start daemon
* Install the latest `diesel_cl`
  - `cargo install diesel_cli -f --features postgres --no-default-features`

Then:

```bash
./run.sh
```
