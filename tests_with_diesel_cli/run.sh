#!/bin/bash
set -xe

export DATABASE_URL=postgres://postgres:postgres@localhost:5432

rm -rf src/schema.rs
rm -rf src/custom_schema.rs

# create a 'default' schema
docker-compose down
docker-compose up -d
sleep 1

diesel migration run
cargo test
rm -rf src/schema.rs
diesel migration revert

# create a custom schema
diesel migration run --config-file custom.diesel.toml
cargo test --features custom
rm -rf src/custom_schema.rs
