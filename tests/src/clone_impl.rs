#[cfg(feature = "postgres")]
use diesel::prelude::*;

// SCENARIO 1: When the SQL type ALREADY HAS Clone (like those gen by diesel_cli)
// Here we simulate a type with Clone already implemented
// We use generics for the SQL type definitions to avoid needing database-specific features
#[cfg(feature = "postgres")]
#[derive(diesel::sql_types::SqlType, Clone)]
pub struct AlreadyHasCloneMapping;

// SCENARIO 2: When the SQL type DOES NOT HAVE Clone (atypical case)
// Here we create a type without Clone
#[cfg(feature = "postgres")]
#[derive(diesel::sql_types::SqlType)]
pub struct NeedsCloneMapping;

// Define enums for each scenario
// SCENARIO 1: Using a SQL type that ALREADY has Clone 
// We don't need to add impl_clone_on_sql_type here since the SQL type already has Clone
#[cfg(feature = "postgres")]
#[derive(Debug, PartialEq, Clone, diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "AlreadyHasCloneMapping")]
pub enum AlreadyHasCloneEnum {
    A,
    B
}

// SCENARIO 2: Using a SQL type that NEEDS Clone
// We need to explicitly request Clone implementation with impl_clone_on_sql_type
#[cfg(feature = "postgres")]
#[derive(Debug, PartialEq, Clone, diesel_derive_enum::DbEnum)]
#[db_enum(existing_type_path = "NeedsCloneMapping")]
#[db_enum(impl_clone_on_sql_type)]
pub enum NeedsCloneEnum {
    X,
    Y
}

// Simple test to verify that when a SQL type already has Clone,
// we can use it directly (diesel_cli case)
#[cfg(feature = "postgres")]
#[test]
fn test_already_has_clone() {
    // This test verifies that when diesel_cli has already implemented Clone on the 
    // SQL type, we can use it without adding impl_clone_on_sql_type
    
    // Create an instance of the SQL type
    let sql_type = AlreadyHasCloneMapping;
    
    // Clone it (this line will fail to compile if Clone is not implemented)
    let _cloned = sql_type.clone();
    
    // Verify the enum works too
    let enum_val = AlreadyHasCloneEnum::A;
    let _cloned_enum = enum_val.clone();
}

// Test that we can implement Clone for a SQL type that needs it
// using the impl_clone_on_sql_type attribute
#[cfg(feature = "postgres")]
#[test]
fn test_needs_clone() {
    // This test verifies that when no Clone impl exists for the SQL type,
    // we can add it using impl_clone_on_sql_type
    
    // Create an instance of the SQL type
    let sql_type = NeedsCloneMapping;
    
    // Clone it (this line will fail to compile if Clone is not implemented)
    // If this test compiles, it means impl_clone_on_sql_type is working correctly
    let _cloned = sql_type.clone();
    
    // Verify the enum works too
    let enum_val = NeedsCloneEnum::X;
    let _cloned_enum = enum_val.clone();
}