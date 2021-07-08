use diesel::prelude::*;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::common::get_connection;

#[derive(Debug, PartialEq, diesel_derive_enum::DbEnum)]
#[DieselType = "Some_Internal_Type"]
#[DieselExistingType = "Some_Internal_Type_Pg"]
pub enum SomeEnum {
    #[db_rename = "mod"]
    Mod,
    #[db_rename = "type"]
    typo,
    #[db_rename = "with spaces"]
    WithASpace,
}

#[derive(diesel::sql_types::SqlType)]
#[postgres(type_name = "Some_External_Type")]
pub struct Some_Internal_Type_Pg;
#[cfg(feature = "postgres")]
table! {
    use diesel::sql_types::Integer;
    use super::Some_Internal_Type_Pg;
    test_rename {
        id -> Integer,
        renamed -> Some_Internal_Type_Pg,
    }
}
#[cfg(not(feature = "postgres"))]
table! {
    use diesel::sql_types::Integer;
    use super::Some_Internal_Type;
    test_rename {
        id -> Integer,
        renamed -> Some_Internal_Type,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test_rename"]
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
    let connection = get_connection();
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
        .get_results(&connection)
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
    let connection = get_connection();
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
        .execute(&connection)
        .unwrap();
    let inserted = test_rename::table.load::<TestRename>(&connection).unwrap();
    assert_eq!(data, inserted);
}
