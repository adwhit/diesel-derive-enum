use diesel::prelude::*;

use diesel_derive_enum::DbEnum;

#[derive(Debug, PartialEq, DbEnum, Clone)]
#[cfg_attr(feature = "postgres", DieselTypePath = "MyEnumPgMapping")]
pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}

#[cfg(feature = "postgres")]
#[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
#[diesel(postgres_type(name = "my_enum"))]
pub struct MyEnumPgMapping;

#[cfg(feature = "postgres")]
table! {
    use diesel::sql_types::Integer;
    use super::MyEnumPgMapping;
    test_simple {
        id -> Integer,
        my_enum -> MyEnumPgMapping,
    }
}

#[cfg(any(feature = "mysql", feature = "sqlite"))]
table! {
    use diesel::sql_types::Integer;
    use super::MyEnumMapping;
    test_simple {
        id -> Integer,
        my_enum -> MyEnumMapping,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[diesel(table_name = test_simple)]
pub struct Simple {
    pub id: i32,
    pub my_enum: MyEnum,
}

#[cfg(feature = "postgres")]
pub fn get_connection() -> PgConnection {
    use diesel::connection::SimpleConnection;
    let database_url =
        ::std::env::var("PG_TEST_DATABASE_URL").expect("Env var PG_TEST_DATABASE_URL not set");
    let mut conn = PgConnection::establish(&database_url)
        .expect(&format!("Failed to connect to {}", database_url));
    conn.batch_execute("SET search_path TO pg_temp;").unwrap();
    conn
}

#[cfg(feature = "mysql")]
pub fn get_connection() -> MysqlConnection {
    let database_url = ::std::env::var("MYSQL_TEST_DATABASE_URL")
        .expect("Env var MYSQL_TEST_DATABASE_URL not set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Failed to connect to {}", database_url))
}

#[cfg(feature = "sqlite")]
pub fn get_connection() -> SqliteConnection {
    let database_url = ":memory:";
    SqliteConnection::establish(&database_url)
        .expect(&format!("Failed to connect to {}", database_url))
}

pub fn sample_data() -> Vec<Simple> {
    vec![
        Simple {
            id: 1,
            my_enum: MyEnum::Foo,
        },
        Simple {
            id: 2,
            my_enum: MyEnum::BazQuxx,
        },
        Simple {
            id: 33,
            my_enum: MyEnum::Bar,
        },
        Simple {
            id: 44,
            my_enum: MyEnum::Foo,
        },
        Simple {
            id: 555,
            my_enum: MyEnum::Foo,
        },
    ]
}

#[cfg(feature = "postgres")]
pub fn create_table(conn: &mut PgConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
        CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');
        CREATE TABLE test_simple (
            id SERIAL PRIMARY KEY,
            my_enum my_enum NOT NULL
        );
    "#,
    )
    .unwrap();
}

#[cfg(feature = "mysql")]
pub fn create_table(conn: &mut MysqlConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
        CREATE TEMPORARY TABLE IF NOT EXISTS test_simple (
            id SERIAL PRIMARY KEY,
            my_enum enum('foo', 'bar', 'baz_quxx') NOT NULL
        );
    "#,
    )
    .unwrap();
}

#[cfg(feature = "sqlite")]
pub fn create_table(conn: &mut SqliteConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
        CREATE TABLE test_simple (
            id SERIAL PRIMARY KEY,
            my_enum TEXT CHECK(my_enum IN ('foo', 'bar', 'baz_quxx')) NOT NULL
        );
    "#,
    )
    .unwrap();
}
