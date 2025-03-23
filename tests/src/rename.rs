use diesel::prelude::*;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::common::get_connection;

#[derive(Debug, PartialEq, diesel_derive_enum::DbEnum)]
#[db_enum(diesel_type = "Some_Internal_Type", pg_type = "Some_External_Type")]
pub enum SomeEnum {
    #[db_enum(rename = "mod")]
    Mod,
    #[db_enum(rename = "type")]
    typo,
    #[db_enum(rename = "with spaces")]
    WithASpace,
}

table! {
    use diesel::sql_types::Integer;
    use super::Some_Internal_Type;
    test_rename {
        id -> Integer,
        renamed -> Some_Internal_Type,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = test_rename)]
struct TestRename {
    id: i32,
    renamed: SomeEnum,
}

#[test]
#[cfg(feature = "postgres")]
fn rename_round_trip() {
    use diesel::connection::SimpleConnection;
    use diesel::insert_into;
    let data = vec![
        TestRename {
            id: 1,
            renamed: SomeEnum::Mod,
        },
        TestRename {
            id: 2,
            renamed: SomeEnum::WithASpace,
        },
    ];
    let connection = &mut get_connection();
    connection
        .batch_execute(
            r#"
        CREATE TYPE "Some_External_Type" AS ENUM ('mod', 'type', 'with spaces');
        CREATE TABLE test_rename (
            id SERIAL PRIMARY KEY,
            renamed "Some_External_Type" NOT NULL
        );
    "#,
        )
        .unwrap();
    let inserted = insert_into(test_rename::table)
        .values(&data)
        .get_results(connection)
        .unwrap();
    assert_eq!(data, inserted);
}

#[test]
#[cfg(feature = "mysql")]
fn rename_round_trip() {
    use diesel::connection::SimpleConnection;
    use diesel::insert_into;
    let data = vec![
        TestRename {
            id: 1,
            renamed: SomeEnum::Mod,
        },
        TestRename {
            id: 2,
            renamed: SomeEnum::WithASpace,
        },
    ];
    let connection = &mut get_connection();
    connection
        .batch_execute(
            r#"
        CREATE TEMPORARY TABLE IF NOT EXISTS test_rename (
            id SERIAL PRIMARY KEY,
            renamed enum('mod', 'type', 'with spaces') NOT NULL
        );
    "#,
        )
        .unwrap();
    insert_into(test_rename::table)
        .values(&data)
        .execute(connection)
        .unwrap();
    let inserted = test_rename::table.load::<TestRename>(connection).unwrap();
    assert_eq!(data, inserted);
}
