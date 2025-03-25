local-tests:
    cargo test

    cd tests && cargo test --features postgres --no-run
    cd tests && cargo test --features mysql --no-run
    cd tests && cargo test --features sqlite --no-run

    docker-compose -f tests/docker-compose.yaml up -d
    sleep 2
    
    cd tests && PG_TEST_DATABASE_URL="postgres://postgres:postgres@localhost:54321" cargo test --features postgres
    cd tests && MYSQL_TEST_DATABASE_URL="mysql://root:mysql@127.0.0.1:3306/test" cargo test --features mysql
    cd tests && cargo test --features sqlite

reset:
    docker-compose -f tests/docker-compose.yaml down
