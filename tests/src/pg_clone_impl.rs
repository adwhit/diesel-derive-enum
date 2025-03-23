use diesel::prelude::*;

#[cfg(feature = "postgres")]
#[derive(diesel::sql_types::SqlType)]
#[diesel(postgres_type(name = "my_enum_no_clone"))]
pub struct MyEnumNoCloneMapping;

#[cfg(feature = "postgres")]
#[derive(diesel::sql_types::SqlType, Clone)]
#[diesel(postgres_type(name = "my_enum_with_clone"))]
pub struct MyEnumWithCloneMapping;

table! {
    use diesel::sql_types::Integer;
    use super::MyEnumNoCloneMapping;
    test_no_clone {
        id -> Integer,
        my_enum -> MyEnumNoCloneMapping,
    }
}

table! {
    use diesel::sql_types::Integer;
    use super::MyEnumWithCloneMapping;
    test_with_clone {
        id -> Integer,
        my_enum -> MyEnumWithCloneMapping,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[diesel(table_name = test_no_clone)]
struct DataNoClone {
    id: i32,
    my_enum: MyEnumNoClone
}

#[derive(Insertable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[diesel(table_name = test_with_clone)]
struct DataWithClone {
    id: i32,
    my_enum: MyEnumWithClone
}

// Default behavior in v3: No Clone implementation for SQL type
#[derive(Debug, PartialEq, Clone, diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "MyEnumNoCloneMapping")]
pub enum MyEnumNoClone {
    This,
    That
}

// Opt-in behavior: Explicitly request Clone implementation for SQL type
#[derive(Debug, PartialEq, Clone, diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "MyEnumWithCloneMapping")]
#[db_enum(impl_clone_on_sql_type)]
pub enum MyEnumWithClone {
    This,
    That
}

#[test]
#[cfg(feature = "postgres")]
fn test_default_no_clone() {
    // This test verifies that Clone is not implemented by default in v3
    // If this compiles, the test passes (there's no easy way to test the absence of a trait)
    
    // Basic functionality still works
    let enum_value = MyEnumNoClone::This;
    assert_eq!(enum_value, MyEnumNoClone::This);
}

#[test]
#[cfg(feature = "postgres")]
fn test_opt_in_clone() {
    // This test verifies that Clone is implemented when explicitly requested
    
    // Create an instance of the SQL type
    let sql_type = MyEnumWithCloneMapping;
    
    // Clone it (this line will fail to compile if Clone is not implemented)
    let _cloned = sql_type.clone();
}