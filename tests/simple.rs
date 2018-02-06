#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
pub extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;

use diesel::prelude::*;
use diesel::insert_into;

#[derive(Debug, PartialEq, DbEnum, Clone)]
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

#[derive(Insertable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[table_name = "test_simple"]
struct Simple {
    id: i32,
    my_enum: MyEnum,
}

#[cfg(feature = "postgres")]
pub fn get_connection() -> PgConnection {
    let database_url =
        std::env::var("TEST_DATABASE_URL").expect("Env var TEST_DATABASE_URL not set");
    PgConnection::establish(&database_url).expect(&format!("Failed to connect to {}", database_url))
}

#[cfg(feature = "sqlite")]
pub fn get_connection() -> SqliteConnection {
    let database_url = ":memory:";
    SqliteConnection::establish(&database_url)
        .expect(&format!("Failed to connect to {}", database_url))
}

fn sample_data() -> Vec<Simple> {
    vec![
        Simple {
            id: 1,
            my_enum: MyEnum::Foo,
        },
        Simple {
            id: 2,
            my_enum: MyEnum::BazQuxx,
        },
        Simple {
            id: 33,
            my_enum: MyEnum::Bar,
        },
        Simple {
            id: 44,
            my_enum: MyEnum::Foo,
        },
        Simple {
            id: 555,
            my_enum: MyEnum::Foo,
        },
    ]
}

#[cfg(feature = "postgres")]
fn create_table(conn: &PgConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
        DROP TYPE IF EXISTS my_enum;
        CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');
        CREATE TEMP TABLE IF NOT EXISTS test_simple (
            id SERIAL PRIMARY KEY,
            my_enum my_enum NOT NULL
        );
    "#,
    ).unwrap();
}

#[cfg(feature = "postgres")]
fn drop_table(conn: &PgConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
            DROP TABLE test_simple;
            DROP TYPE my_enum;
         "#,
    ).unwrap();
}

#[cfg(feature = "sqlite")]
fn create_table(conn: &SqliteConnection) {
    conn
        .execute(
            r#"
        CREATE TABLE test_simple (
            id SERIAL PRIMARY KEY,
            my_enum TEXT CHECK(my_enum IN ('foo', 'bar', 'baz_quxx')) NOT NULL
        );
    "#,
        )
        .unwrap();
}

#[cfg(feature = "sqlite")]
fn drop_table(_conn: &SqliteConnection) {
    // no-op
}

#[test]
#[cfg(any(feature = "sqlite", feature = "postgres"))]
fn enum_round_trip() {
    let connection = get_connection();
    create_table(&connection);
    let data = sample_data();
    let ct = insert_into(test_simple::table)
        .values(&data)
        .execute(&connection)
        .unwrap();
    assert_eq!(data.len(), ct);
    let items = test_simple::table.load::<Simple>(&connection).unwrap();
    assert_eq!(data, items);
    drop_table(&connection);
}

#[test]
#[cfg(any(feature = "sqlite", feature = "postgres"))]
fn filter_by_enum() {
    use test_simple::dsl::*;
    // TODO this is one ugly hack (it stops us creating same table twice at once)
    std::thread::sleep(std::time::Duration::from_secs(1));
    let connection = get_connection();
    create_table(&connection);
    let data = sample_data();
    let ct = insert_into(test_simple)
        .values(&data)
        .execute(&connection)
        .unwrap();
    assert_eq!(data.len(), ct);
    let results = test_simple
        .filter(my_enum.eq(MyEnum::Foo))
        .limit(2)
        .load::<Simple>(&connection)
        .unwrap();
    assert_eq!(
        results,
        vec![
            Simple {
                id: 1,
                my_enum: MyEnum::Foo,
            },
            Simple {
                id: 44,
                my_enum: MyEnum::Foo,
            },
        ]
    );
    drop_table(&connection);
}

#[test]
#[cfg(feature = "sqlite")]
fn sqlite_invalid_enum() {
    let connection = get_connection();
    let data = sample_data();
    connection
        .execute(
            r#"
        CREATE TABLE test_simple (
            id SERIAL PRIMARY KEY,
            my_enum TEXT CHECK(my_enum IN ('food', 'bar', 'baz_quxx')) NOT NULL
        );
    "#,
        )
        .unwrap();
    if let Err(e) = insert_into(test_simple::table)
        .values(&data)
        .execute(&connection)
    {
        let err = format!("{}", e);
        assert!(err.contains("CHECK constraint failed"));
    } else {
        panic!("should have failed to insert")
    }
    drop_table(&connection);
}

// test snakey naming - should compile and not clobber above definitions
// (but we won't actually bother round-tripping)

#[derive(Debug, PartialEq, DbEnum)]
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
