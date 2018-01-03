#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;

#[derive(Debug, PartialEq, PgEnum)]
#[PgType = "MyType"]
pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}

table! {
    use diesel::types::Integer;
    use super::MyType;
    test {
        id -> Integer,
        my_enum -> MyType,
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
