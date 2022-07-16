use diesel::prelude::*;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::common::get_connection;

#[derive(diesel::sql_types::SqlType)]
#[diesel(postgres_type(name = "Some_External_Type"))]
pub struct Some_Internal_Type_Pg;

#[derive(Debug, PartialEq, diesel_derive_enum::DbEnum)]
#[cfg_attr(
    any(feature = "mysql", feature = "sqlite"),
    DieselType = "Some_Internal_Type"
)]
#[cfg_attr(feature = "postgres", DieselTypePath = "Some_Internal_Type_Pg")]
pub enum SomeEnum {
    #[db_rename = "mod"]
    Mod,
    #[db_rename = "type"]
    typo,
    #[db_rename = "with spaces"]
    WithASpace,
}

#[cfg(feature = "postgres")]
table! {
    use diesel::sql_types::Integer;
    use super::Some_Internal_Type_Pg;
    test_rename {
        id -> Integer,
        renamed -> Some_Internal_Type_Pg,
    }
}
#[cfg(any(feature = "mysql", feature = "sqlite"))]
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
