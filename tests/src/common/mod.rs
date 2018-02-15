use diesel::prelude::*;

#[derive(Debug, PartialEq, DbEnum, Clone)]
pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}

table! {
    use diesel::sql_types::Integer;
    use super::MyEnumMapping;
    test_simple {
        id -> Integer,
        my_enum -> MyEnumMapping,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[table_name = "test_simple"]
pub struct Simple {
    pub id: i32,
    pub my_enum: MyEnum,
}

#[cfg(feature = "postgres")]
pub fn get_connection() -> PgConnection {
    let database_url =
        ::std::env::var("TEST_DATABASE_URL").expect("Env var TEST_DATABASE_URL not set");
    PgConnection::establish(&database_url).expect(&format!("Failed to connect to {}", database_url))
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
pub fn create_table(conn: &PgConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
        DROP TYPE IF EXISTS my_enum;
        CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');
        CREATE TEMP TABLE IF NOT EXISTS test_simple (
            id SERIAL PRIMARY KEY,
            my_enum my_enum NOT NULL
        );
    "#,
    ).unwrap();
}

#[cfg(feature = "postgres")]
pub fn drop_table(conn: &PgConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
            DROP TABLE test_simple;
            DROP TYPE my_enum;
         "#,
    ).unwrap();
}

#[cfg(feature = "sqlite")]
pub fn create_table(conn: &SqliteConnection) {
    conn.execute(
        r#"
        CREATE TABLE test_simple (
            id SERIAL PRIMARY KEY,
            my_enum TEXT CHECK(my_enum IN ('foo', 'bar', 'baz_quxx')) NOT NULL
        );
    "#,
    ).unwrap();
}

#[cfg(feature = "sqlite")]
pub fn drop_table(_conn: &SqliteConnection) {
    // no-op
}
