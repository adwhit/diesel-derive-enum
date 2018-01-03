# diesel-derive-enum
[![Build Status](https://travis-ci.org/adwhit/diesel-derive-enum.svg?branch=master)](https://travis-ci.org/adwhit/diesel-derive-enum)

This crate allows one to automatically derive the Diesel boilerplate necessary
to use Rust enums directly with Postgres databases.

It is a fairly literal translation of [this code](https://github.com/diesel-rs/diesel/blob/8f8dd92135a788c7d0f2c5202dcb4d05339a0cc1/diesel_tests/tests/custom_types.rs) from the Diesel test suite.

Example usage: 

```rust
// define your enum
#[derive(PgEnum)]
#[PgType = "my_type"]  // This is the name of the type within the database
pub enum MyEnum {      // All enum variants must be fieldless
    Foo,
    Bar,
    BazQuxx,
}

// define your table
table! {
    use diesel::types::Integer;
    use super::MyType;
    my_table {
        id -> Integer,
        my_enum -> MyType, // A Diesel type "MyType" has been created corresponding to my_type
    }
}

// define a struct with which to populate/query the table
#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "my_table"]
struct  MyRow {
    id: i32,
    my_enum: MyEnum,
}
```

SQL to create corresponding table:

```sql
CREATE TYPE my_type AS ENUM ('foo', 'bar', 'baz_quxx');
-- Note: the postgres ENUM values must correspond to snake_cased Rust enum variant names
CREATE TABLE my_table (
  id SERIAL PRIMARY KEY,
  my_enum my_type NOT NULL
);
```

Now we can insert and retrieve MyEnum directly:

```rust
let data = vec![
    MyRow {
        id: 1,
        my_enum: MyEnum::Foo,
    },
    MyRow {
        id: 2,
        my_enum: MyEnum::BazQuxx,
    },
];
let connection = PgConnection::establish(/*...*/).unwrap();
let inserted = insert_into(my_table::table)
    .values(&data)
    .get_results(&connection)
    .unwrap();
assert_eq!(data, inserted);
```

See [this test]("tests/lib.rs") for a full working example.
