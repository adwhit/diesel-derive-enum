#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
pub extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;

use ::diesel::prelude::*;
use ::diesel::insert_into;
use diesel::connection::SimpleConnection;

pub fn connection() -> PgConnection {
    let database_url =
        std::env::var("TEST_DATABASE_URL").expect("Env var TEST_DATABASE_URL not set");
    PgConnection::establish(&database_url).expect(&format!("Failed to connect to {}", database_url))
}

#[derive(Debug, PartialEq, PgEnum)]
#[PgType = "Just_Whatever"]
#[DieselType = "Some_Ugly_Renaming"]
pub enum RenameMe {
    #[pg_rename = "mod"] Mod,
    #[pg_rename = "type"] typo,
    #[pg_rename = "with spaces"] WithASpace,
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
fn rename_round_trip() {
    let data = vec![
        TestRename {
            id: 1,
            renamed: RenameMe::Mod,
        },
        TestRename {
            id: 2,
            renamed: RenameMe::WithASpace
        },
    ];
    let connection = connection();
    connection
        .batch_execute(
            r#"
        DROP TYPE IF EXISTS Just_Whatever;
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
