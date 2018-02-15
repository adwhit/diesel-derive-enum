use diesel::prelude::*;
use diesel::insert_into;

use common::*;

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

#[cfg(feature = "postgres")]
pub fn create_table(conn: &PgConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
        DROP TYPE IF EXISTS my_enum;
        CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');
        CREATE TEMP TABLE IF NOT EXISTS test_nullable (
            id SERIAL PRIMARY KEY,
            my_enum my_enum
        );
    "#,
    ).unwrap();
}

#[cfg(feature = "sqlite")]
pub fn create_table(conn: &SqliteConnection) {
    conn.execute(
        r#"
        CREATE TABLE test_nullable (
            id SERIAL PRIMARY KEY,
            my_enum TEXT CHECK(my_enum IN ('foo', 'bar', 'baz_quxx'))
        );
    "#,
    ).unwrap();
}

#[cfg(feature = "postgres")]
pub fn drop_table(conn: &PgConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
            DROP TABLE test_nullable;
            DROP TYPE my_enum;
         "#,
    ).unwrap();
}

#[test]
#[cfg(any(feature = "sqlite", feature = "postgres"))]
fn nullable_enum_round_trip() {
    let connection = get_connection();
    create_table(&connection);
    let data = vec![
        Nullable {
            id: 1,
            my_enum: None,
        },
        Nullable {
            id: 2,
            my_enum: Some(MyEnum::Bar),
        },
    ];
    let ct = insert_into(test_nullable::table)
        .values(&data)
        .execute(&connection)
        .unwrap();
    assert_eq!(data.len(), ct);
    let items = test_nullable::table.load::<Nullable>(&connection).unwrap();
    assert_eq!(data, items);
    drop_table(&connection);
}

#[test]
#[cfg(any(feature = "sqlite", feature = "postgres"))]
fn not_nullable_enum_round_trip() {
    // TODO this is one ugly hack (it stops us creating same table twice at once)
    std::thread::sleep(std::time::Duration::from_secs(1));
    let connection = get_connection();
    create_table(&connection);
    let data = vec![
        MaybeNullable {
            id: 1,
            my_enum: MyEnum::Foo,
        },
        MaybeNullable {
            id: 2,
            my_enum: MyEnum::BazQuxx,
        },
    ];
    let ct = insert_into(test_nullable::table)
        .values(&data)
        .execute(&connection)
        .unwrap();
    assert_eq!(data.len(), ct);
    drop_table(&connection);
}
