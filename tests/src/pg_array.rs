use diesel::insert_into;
use diesel::prelude::*;

use crate::common::*;

pub fn create_table(conn: &mut PgConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
        CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');
        CREATE TABLE test_array (
            id SERIAL PRIMARY KEY,
            my_enum_arr my_enum[] NOT NULL
        );
    "#,
    )
    .unwrap();
}

#[test]
fn enum_query() {
    let connection = &mut get_connection();
    create_table(connection);
    let data_item = TestArray {
        id: 1,
        my_enum_arr: vec![MyEnum::Foo],
    };
    let data = vec![data_item];
    let ct = insert_into(test_array::table)
        .values(&data)
        .execute(connection)
        .unwrap();
    assert_eq!(data.len(), ct);
    let item = test_array::table
        .find(1)
        .get_results::<TestArray>(connection)
        .unwrap();
    assert_eq!(data, item);
}

table! {
    use diesel::sql_types::{Integer, Array};
    use super::MyEnumMapping;
    test_array {
        id -> Integer,
        my_enum_arr -> Array<MyEnumMapping>,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[diesel(table_name = test_array)]
struct TestArray {
    id: i32,
    my_enum_arr: Vec<MyEnum>,
}
