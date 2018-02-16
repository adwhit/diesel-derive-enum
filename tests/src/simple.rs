use diesel::prelude::*;
use diesel::insert_into;

use common::*;

#[test]
#[cfg(any(feature = "sqlite", feature = "postgres"))]
fn enum_round_trip() {
    let connection = get_connection();
    create_table(&connection);
    let data = sample_data();
    let ct = insert_into(test_simple::table)
        .values(&data)
        .execute(&connection)
        .unwrap();
    assert_eq!(data.len(), ct);
    let items = test_simple::table.load::<Simple>(&connection).unwrap();
    assert_eq!(data, items);
}

#[test]
#[cfg(any(feature = "sqlite", feature = "postgres"))]
fn filter_by_enum() {
    use common::test_simple::dsl::*;
    let connection = get_connection();
    create_table(&connection);
    let data = sample_data();
    let ct = insert_into(test_simple)
        .values(&data)
        .execute(&connection)
        .unwrap();
    assert_eq!(data.len(), ct);
    let results = test_simple
        .filter(my_enum.eq(MyEnum::Foo))
        .limit(2)
        .load::<Simple>(&connection)
        .unwrap();
    assert_eq!(
        results,
        vec![
            Simple {
                id: 1,
                my_enum: MyEnum::Foo,
            },
            Simple {
                id: 44,
                my_enum: MyEnum::Foo,
            },
        ]
    );
}

#[test]
#[cfg(feature = "sqlite")]
fn sqlite_invalid_enum() {
    let connection = get_connection();
    let data = sample_data();
    connection
        .execute(
            r#"
        CREATE TABLE test_simple (
            id SERIAL PRIMARY KEY,
            my_enum TEXT CHECK(my_enum IN ('food', 'bar', 'baz_quxx')) NOT NULL
        );
    "#,
        )
        .unwrap();
    if let Err(e) = insert_into(test_simple::table)
        .values(&data)
        .execute(&connection)
    {
        let err = format!("{}", e);
        assert!(err.contains("CHECK constraint failed"));
    } else {
        panic!("should have failed to insert")
    }
}

// test snakey naming - should compile and not clobber above definitions
// (but we won't actually bother round-tripping)

#[derive(Debug, PartialEq, DbEnum)]
pub enum my_enum {
    foo,
    bar,
    bazQuxx,
}

table! {
    use diesel::sql_types::Integer;
    use super::my_enumMapping;
    test_snakey {
        id -> Integer,
        my_enum -> my_enumMapping,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test_snakey"]
struct test_snake {
    id: i32,
    my_enum: my_enum,
}
