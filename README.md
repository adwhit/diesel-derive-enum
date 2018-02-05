# diesel-derive-enum
[![crates.io](https://img.shields.io/crates/v/diesel-derive-enum.svg)](https://crates.io/crates/diesel-derive-enum)
[![Build Status](https://travis-ci.org/adwhit/diesel-derive-enum.svg?branch=master)](https://travis-ci.org/adwhit/diesel-derive-enum)

This crate allows one to automatically derive the Diesel boilerplate necessary
to use Rust enums directly with `Postgres` and `sqlite` databases.

It is a fairly literal translation of [this code](https://github.com/diesel-rs/diesel/blob/8f8dd92135a788c7d0f2c5202dcb4d05339a0cc1/diesel_tests/tests/custom_types.rs) from the Diesel test suite.

v0.3+ requires Diesel 1.1+. For Diesel 1.0 use v0.2.2.

### Example usage (Postgres): 

```toml
# Cargo.toml

[dependencies]
diesel-derive-enum = { version = "0.4", features = ["postgres"] }
```

```rust

// define your enum
#[derive(DbEnum)]
pub enum MyEnum {
    Foo,  // All variants must be fieldless
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

### sqlite

`sqlite` is untyped. [Yes, really.](https://dba.stackexchange.com/questions/106364/text-string-stored-in-sqlite-integer-column). You can store any kind of data in any column and it won't complain. How do we get some nice static checking then? Well... you can't, really, but you can emulate it by add a `CHECK` to your column definition like so:

```sql
CREATE TABLE test_simple (
    id SERIAL PRIMARY KEY,
    some_enum TEXT CHECK(my_enum IN ('foo', 'bar', 'baz_quxx')) NOT NULL
);
```

If you substitute this snippet into the above example, all will be well (and make sure to edit your `Cargo.toml` to include `features = ["sqlite"]`). Trivia: the `TEXT` type annotation isn't even frickin used and you could substitute `MY_FAVOURITE_SHINY_TYPE` in it's place and it would still work.

Note that it will still be possible to insert other strings (or whatever) into the column 'by hand', though it will be a type error should you attempt to do so through `diesel`. If you attempt to retreive some other invalid text as an enum, `diesel` will error at the point of deserialization.

### What's up with the naming?

Diesel maintains a set of internal types which correspond one-to-one to the types available in various relational databases. Each internal type then maps to some kind of Rust native type. e.g. `diesel::types::Integer` maps to `i32`. So, when we create a new type in Postgres with `CREATE TYPE ...`, we must also create a corresponding type in Diesel, and then create a mapping to some native Rust type (our enum). Hence there are three types we need to be aware of.

By default, the Postgres and Diesel internal types are inferred from the name of the Rust enum. Specifically, we assume `MyEnum` corresponds to `my_enum` in Postgres and `MyEnumMapping` in Diesel. (The Diesel type is created by the plugin, the Postgres type must be created in SQL).

These defaults can be overridden with the attributes `#[PgType = "..."]` and `#[DieselType = "..."]`. (The `PgType` annotation has no effect on `sqlite`).

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
