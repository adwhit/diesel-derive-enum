use diesel::prelude::*;

#[cfg(feature = "postgres")]
#[derive(diesel::sql_types::SqlType)]
#[diesel(postgres_type(name = "my_remote_enum"))]
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
    my_enum: MyRemoteEnum,
}

#[derive(Debug, PartialEq, Clone, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "MyRemoteEnumMapping"]
pub enum MyRemoteEnum {
    This,
    That,
}

#[test]
fn clone_impl_on_sql_type() {
    let x = MyRemoteEnumMapping {};
    let _ = x.clone();
}
