use diesel::prelude::*;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::common::get_connection;

#[derive(Debug, PartialEq, diesel_derive_enum::DbEnum)]
#[PgSchema = "schema"]
pub enum SomeEnum {
    One,
    Two,
}

table! {
    use diesel::sql_types::Integer;
    use super::SomeEnumMapping;
    test_schema {
        id -> Integer,
        enum_ -> SomeEnumMapping,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test_schema"]
struct TestSchema {
    id: i32,
    enum_: SomeEnum,
}

#[test]
#[cfg(feature = "postgres")]
fn schema_round_trip() {
    use diesel::connection::SimpleConnection;
    use diesel::insert_into;
    let data = vec![
        TestSchema {
            id: 1,
            enum_: SomeEnum::One,
        },
        TestSchema {
            id: 2,
            enum_: SomeEnum::Two,
        },
    ];
    let connection = get_connection();
    connection
        .batch_execute(
            r#"
        CREATE SCHEMA "schema";
        CREATE TYPE "schema"."some_enum" AS ENUM ('one', 'two');
        CREATE TABLE test_schema (
            id SERIAL PRIMARY KEY,
            enum_ "schema"."some_enum" NOT NULL
        );
    "#,
        )
        .unwrap();
    let inserted = insert_into(test_schema::table)
        .values(&data)
        .get_results(&connection)
        .unwrap();
    assert_eq!(data, inserted);
}
