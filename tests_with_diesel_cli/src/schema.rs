// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "my_enum"))]
    pub struct MyEnum;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MyEnum;

    simple (id) {
        id -> Int4,
        some_value -> MyEnum,
    }
}
