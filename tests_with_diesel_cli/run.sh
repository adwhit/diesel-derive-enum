#!/bin/bash
set -xe

export DATABASE_URL=postgres://postgres:postgres@localhost:5432

rm -f src/schema.rs
rm -f src/custom_schema.rs

# create a 'default' schema
docker-compose down
docker-compose up -d
sleep 2

diesel migration run
cargo test
diesel migration revert
rm src/schema.rs

# create a custom schema
diesel migration run --config-file custom.diesel.toml
cargo test --features custom
rm src/custom_schema.rs
