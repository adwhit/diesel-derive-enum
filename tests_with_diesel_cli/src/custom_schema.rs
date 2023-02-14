// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    use crate::with_custom_schema::export::MyEnum;

    simple (id) {
        id -> Int4,
        some_value -> MyEnum,
    }
}
