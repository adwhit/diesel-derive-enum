#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;

#[derive(Debug, PartialEq, PgEnum)]
pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}

table! {
    use diesel::types::Integer;
    use super::MyEnumMapping;
    test {
        id -> Integer,
        my_enum -> MyEnumMapping,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test"]
struct Test {
    id: i32,
    my_enum: MyEnum,
}

#[test]
fn enum_round_trip() {
    use diesel::insert_into;
    use diesel::connection::SimpleConnection;
    use diesel::prelude::*;
    let data = vec![
        Test {
            id: 1,
            my_enum: MyEnum::Foo,
        },
        Test {
            id: 2,
            my_enum: MyEnum::BazQuxx,
        },
    ];
    let database_url = std::env::var("TEST_DATABASE_URL").expect("Env var TEST_DATABASE_URL not set");
    let connection = PgConnection::establish(&database_url)
        .expect(&format!("Failed to connect to {}", database_url));
    connection
        .batch_execute(r#"
        DROP TYPE IF EXISTS my_type;
        CREATE TYPE my_type AS ENUM ('foo', 'bar', 'baz_quxx');
        CREATE TABLE IF NOT EXISTS test (
            id SERIAL PRIMARY KEY,
            my_enum my_type NOT NULL
        );
    "#,
        )
        .unwrap();

    let inserted = insert_into(test::table)
        .values(&data)
        .get_results(&connection)
        .unwrap();
    assert_eq!(data, inserted);
    connection
        .batch_execute(r#"
            DROP TABLE test;
            DROP TYPE my_type;
         "#).unwrap();
}


// snakey naming - should compile and not clobber above definitions

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
    #[pg_rename = "mod"]
    Mod,
    #[pg_rename = "type"]
    Typo
}

table! {
    use diesel::types::Integer;
    use super::Some_Ugly_Renaming;
    test_ugly {
        id -> Integer,
        my_enum -> Some_Ugly_Renaming,
    }
}
