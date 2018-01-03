#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;

use diesel::prelude::*;

#[derive(Debug, PartialEq, PgEnum)]
pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}

table! {
    use diesel::types::Integer;
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

fn connection() -> PgConnection {
    let database_url =
        std::env::var("TEST_DATABASE_URL").expect("Env var TEST_DATABASE_URL not set");
    PgConnection::establish(&database_url).expect(&format!("Failed to connect to {}", database_url))
}

#[test]
fn enum_round_trip() {
    use diesel::insert_into;
    use diesel::connection::SimpleConnection;
    use diesel::prelude::*;
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
        DROP TYPE IF EXISTS my_type;
        CREATE TYPE my_type AS ENUM ('foo', 'bar', 'baz_quxx');
        CREATE TEMP TABLE IF NOT EXISTS test_simple (
            id SERIAL PRIMARY KEY,
            my_enum my_type NOT NULL
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
            DROP TYPE my_type;
         "#,
        )
        .unwrap();
}

// snakey naming - should compile and not clobber above definitions
// (but we won't actually bother round-tripping)

#[derive(Debug, PartialEq, PgEnum)]
pub enum my_enum {
    foo,
    bar,
    bazQuxx,
}

table! {
    use diesel::types::Integer;
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

// renaming

#[derive(Debug, PartialEq, PgEnum)]
#[PgType = "Just_Whatever"]
#[DieselType = "Some_Ugly_Renaming"]
pub enum RenameMe {
    #[pg_rename = "mod"] Mod,
    #[pg_rename = "type"] Typo,
    #[pg_rename = "with spaces"] WithSpaces,
}

table! {
    use diesel::types::Integer;
    use super::Some_Ugly_Renaming;
    test_rename {
        id -> Integer,
        renamed -> Some_Ugly_Renaming,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test_rename"]
struct TestRename {
    id: i32,
    renamed: RenameMe
}

#[test]
fn enum_rename_round_trip() {
    use diesel::insert_into;
    use diesel::connection::SimpleConnection;
    use diesel::prelude::*;
    let data = vec![
        TestRename {
            id: 1,
            renamed: RenameMe::Mod,
        },
        TestRename {
            id: 2,
            renamed: RenameMe::WithSpaces
        },
    ];
    let connection = connection();
    connection
        .batch_execute(
            r#"
        DROP TYPE IF EXISTS my_type;
        CREATE TYPE Just_Whatever AS ENUM ('mod', 'type', 'with spaces');
        CREATE TEMP TABLE IF NOT EXISTS test_rename (
            id SERIAL PRIMARY KEY,
            renamed Just_Whatever NOT NULL
        );
    "#,
        )
        .unwrap();
    let inserted = insert_into(test_rename::table)
        .values(&data)
        .get_results(&connection)
        .unwrap();
    assert_eq!(data, inserted);
    connection
        .batch_execute(
            r#"
            DROP TABLE test_rename;
            DROP TYPE Just_Whatever;
         "#,
        )
        .unwrap();
}
