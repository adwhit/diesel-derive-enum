use diesel::prelude::*;
use common::get_connection;

#[derive(Debug, PartialEq, DbEnum)]
#[PgType = "Just_Whatever"]
#[DieselType = "Some_Ugly_Renaming"]
pub enum RenameMe {
    #[db_rename = "mod"]
    Mod,
    #[db_rename = "type"]
    typo,
    #[db_rename = "with spaces"]
    WithASpace,
}

table! {
    use diesel::sql_types::Integer;
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
    renamed: RenameMe,
}

#[test]
#[cfg(feature = "postgres")]
fn rename_round_trip() {
    use diesel::connection::SimpleConnection;
    use diesel::insert_into;
    let data = vec![
        TestRename {
            id: 1,
            renamed: RenameMe::Mod,
        },
        TestRename {
            id: 2,
            renamed: RenameMe::WithASpace,
        },
    ];
    let connection = get_connection();
    connection
        .batch_execute(
            r#"
        CREATE TYPE Just_Whatever AS ENUM ('mod', 'type', 'with spaces');
        CREATE TABLE test_rename (
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
}

#[test]
#[cfg(feature = "mysql")]
fn rename_round_trip() {
    use diesel::connection::SimpleConnection;
    use diesel::insert_into;
    let data = vec![
        TestRename {
            id: 1,
            renamed: RenameMe::Mod,
        },
        TestRename {
            id: 2,
            renamed: RenameMe::WithASpace,
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

use diesel;

no_arg_sql_function!(last_insert_id, diesel::types::Bigint);
