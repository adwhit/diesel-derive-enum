#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
pub extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;

use diesel::prelude::*;
use diesel::insert_into;
use diesel::connection::SimpleConnection;

#[derive(Debug, PartialEq, PgEnum)]
pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}

table! {
    use diesel::sql_types::Integer;
    use super::MyEnumMapping;
    test_simple {
        id -> Integer,
        my_enum -> MyEnumMapping,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test_simple"]
struct Simple {
    id: i32,
    my_enum: MyEnum,
}

pub fn connection() -> PgConnection {
    let database_url =
        std::env::var("TEST_DATABASE_URL").expect("Env var TEST_DATABASE_URL not set");
    PgConnection::establish(&database_url).expect(&format!("Failed to connect to {}", database_url))
}

#[test]
fn enum_round_trip() {
    let data = vec![
        Simple {
            id: 1,
            my_enum: MyEnum::Foo,
        },
        Simple {
            id: 2,
            my_enum: MyEnum::BazQuxx,
        },
    ];
    let connection = connection();
    connection
        .batch_execute(
            r#"
        DROP TYPE IF EXISTS my_enum;
        CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');
        CREATE TEMP TABLE IF NOT EXISTS test_simple (
            id SERIAL PRIMARY KEY,
            my_enum my_enum NOT NULL
        );
    "#,
        )
        .unwrap();
    let inserted = insert_into(test_simple::table)
        .values(&data)
        .get_results(&connection)
        .unwrap();
    assert_eq!(data, inserted);
    connection
        .batch_execute(
            r#"
            DROP TABLE test_simple;
            DROP TYPE my_enum;
         "#,
        )
        .unwrap();
}

// test snakey naming - should compile and not clobber above definitions
// (but we won't actually bother round-tripping)

#[derive(Debug, PartialEq, PgEnum)]
pub enum my_enum {
    foo,
    bar,
    bazQuxx,
}

table! {
    use diesel::sql_types::Integer;
    use super::my_enumMapping;
    test_snakey {
        id -> Integer,
        my_enum -> my_enumMapping,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test_snakey"]
struct test_snake {
    id: i32,
    my_enum: my_enum,
}

// test nullable compiles

table! {
    use diesel::sql_types::{Integer, Nullable};
    use super::MyEnumMapping;
    test_nullable {
        id -> Integer,
        my_enum -> Nullable<MyEnumMapping>,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test_nullable"]
struct Nullable {
    id: i32,
    my_enum: Option<MyEnum>,
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test_nullable"]
struct MaybeNullable {
    id: i32,
    my_enum: MyEnum,
}
