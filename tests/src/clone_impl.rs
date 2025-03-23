use diesel::prelude::*;

// SCENARIO 1: Using a SQL type that ALREADY has Clone
// We don't need to add impl_clone_on_sql_mapping here since the SQL type already has Clone
#[derive(Debug, PartialEq, Clone, diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "AlreadyHasCloneMapping")]
pub enum AlreadyHasCloneEnum {
    A,
    B,
}

#[derive(diesel::sql_types::SqlType, Clone)]
pub struct AlreadyHasCloneMapping;

// SCENARIO 2: Using a SQL type that NEEDS Clone
#[derive(Debug, PartialEq, Clone, diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "NeedsCloneMapping")]
#[db_enum(impl_clone_on_sql_mapping)]
pub enum NeedsCloneEnum {
    X,
    Y,
}

#[derive(diesel::sql_types::SqlType)]
pub struct NeedsCloneMapping;

#[test]
fn test_already_has_clone() {
    AlreadyHasCloneMapping.clone();
}

#[test]
fn test_needs_clone() {
    NeedsCloneMapping.clone();
}
