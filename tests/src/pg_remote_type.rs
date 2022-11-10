use crate::common::*;
use diesel::prelude::*;

#[cfg(feature = "postgres")]
#[derive(diesel::sql_types::SqlType)]
#[diesel(postgres_type(name = "my_remoate_enum"))]
pub struct MyRemoteEnumMapping;

table! {
    use diesel::sql_types::Integer;
    use super::MyRemoteEnumMapping;
    test_remote {
        id -> Integer,
        my_enum -> MyRemoteEnumMapping,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[diesel(table_name = test_remote)]
struct Data {
    id: i32,
    my_enum: MyRemoteEnum
}


#[derive(Debug, PartialEq, Clone, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "MyRemoteEnumMapping"]
pub enum MyRemoteEnum {
    This,
    That
}

#[test]
fn enum_round_trip() {
    let connection = &mut get_connection();
    use diesel::connection::SimpleConnection;

    connection.batch_execute(
        r#"
        CREATE TYPE my_remote_enum AS ENUM ('foo', 'bar', 'baz_quxx');
        CREATE TABLE test_remote (
            id SERIAL PRIMARY KEY,
            my_enum my_remote_enum NOT NULL
        );
    "#,
    )
    .unwrap();

    create_table(connection);
    let data = Data { id: 123, my_enum: MyRemoteEnum::This };
    let res = diesel::insert_into(test_remote::table)
        .values(&data)
        .get_result(connection)
        .unwrap();
    assert_eq!(data, res);
}
