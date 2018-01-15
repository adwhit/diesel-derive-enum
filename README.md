# diesel-derive-enum
[![crates.io](https://img.shields.io/crates/v/diesel-derive-enum.svg)](https://crates.io/crates/diesel-derive-enum)
[![Build Status](https://travis-ci.org/adwhit/diesel-derive-enum.svg?branch=master)](https://travis-ci.org/adwhit/diesel-derive-enum)

This crate allows one to automatically derive the Diesel boilerplate necessary
to use Rust enums directly with Postgres databases.

It is a fairly literal translation of [this code](https://github.com/diesel-rs/diesel/blob/8f8dd92135a788c7d0f2c5202dcb4d05339a0cc1/diesel_tests/tests/custom_types.rs) from the Diesel test suite.

### Example usage: 

```rust
// define your enum
#[derive(PgEnum)]
pub enum MyEnum {      // All enum variants must be fieldless
    Foo,
    Bar,
    BazQuxx,
}

// define your table
table! {
    use diesel::types::Integer;
    use super::MyEnumMapping;
    my_table {
        id -> Integer,
        some_enum -> MyEnumMapping, // Generated Diesel type - see below for explanation
    }
}

// define a struct with which to populate/query the table
#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "my_table"]
struct  MyRow {
    id: i32,
    some_enum: MyEnum,
}
```

SQL to create corresponding table:

```sql
-- by default the postgres ENUM values correspond to snake_cased Rust enum variant names
CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');

CREATE TABLE my_table (
  id SERIAL PRIMARY KEY,
  some_enum my_enum NOT NULL
);
```

Now we can insert and retrieve MyEnum directly:

```rust
let data = vec![
    MyRow {
        id: 1,
        some_enum: MyEnum::Foo,
    },
    MyRow {
        id: 2,
        some_enum: MyEnum::BazQuxx,
    },
];
let connection = PgConnection::establish(/*...*/).unwrap();
let inserted = insert_into(my_table::table)
    .values(&data)
    .get_results(&connection)
    .unwrap();
assert_eq!(data, inserted);
```

See [this test](tests/simple.rs) for a full working example.

### What's up with the naming?

Diesel maintains a set of internal types which correspond one-to-one to the types available in Postgres (and other databases). Each internal type then maps to some kind of Rust native type. e.g. `diesel::types::Integer` maps to `i32`. So, when we create a new type in Postgres with `CREATE TYPE ...`, we must also create a corresponding type in Diesel, and then create a mapping to some native Rust type (our enum). Hence there are three types we need to be aware of.

By default, the Postgres and Diesel internal types are inferred from the name of the Rust enum. Specifically, we assume `MyEnum` corresponds to `my_enum` in Postgres and `MyEnumMapping` in Diesel. (The Diesel type is created by the plugin, the Postgres type must be created in SQL).

These defaults can be overridden with the attributes `#[PgType = "..."]` and `#[DieselType = "..."]`.

Similarly, we assume that the Postgres ENUM variants are simply the Rust enum variants translated to `snake_case`. These can be renamed with the inline annotation `#[pg_rename = "..."]`.

See [this test](tests/rename.rs) for an example of renaming.

#### `print-schema` and `infer-schema!`

The `print-schema` command (from `diesel_cli`) attempts to connect to an existing DB and generate a correct mapping of Postgres columns to Diesel internal types. If a custom ENUM exists in the database, Diesel will simply assume that the internal mapping type is the ENUM name, Title-cased (e.g. `my_enum` -> `My_enum`). Therefore the derived mapping name must also be corrected with the `DieselType` attribute e.g. `#[DieselType] = "My_enum"]`.

The `infer_schema!` macro works similarly but unfortunately is not yet compatible with this crate.

### License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
