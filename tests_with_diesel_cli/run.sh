#!/bin/bash

export DATABASE_URL=postgres://postgres:postgres@localhost:5432

# create a 'default' schema
docker-compose down
docker-compose up -d
sleep 1
diesel migration run

# create a custom schema
docker-compose down
docker-compose up -d
sleep 1
diesel migration run --config-file custom.diesel.toml

cargo test
