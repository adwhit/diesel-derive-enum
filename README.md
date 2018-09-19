# diesel-derive-enum
[![crates.io](https://img.shields.io/crates/v/diesel-derive-enum.svg)](https://crates.io/crates/diesel-derive-enum)
[![Build Status](https://travis-ci.org/adwhit/diesel-derive-enum.svg?branch=master)](https://travis-ci.org/adwhit/diesel-derive-enum)

This crate automatically derives the Diesel boilerplate necessary
to use Rust enums directly with `PostgreSQL`, `MySQL` and `sqlite` databases.

Requires diesel v1.1+.

### Example usage:

Cargo.toml:
```toml
[dependencies]
diesel-derive-enum = { version = "0.4", features = ["..."] } # "postgres", "mysql" or "sqlite"
```

Rust:
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

SQL:

Postgres -
```sql
-- by default the postgres ENUM values correspond to snake_cased Rust enum variant names
CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');

CREATE TABLE my_table (
  id SERIAL PRIMARY KEY,
  some_enum my_enum NOT NULL
);
```
MySQL -
```sql
CREATE TABLE my_table (
    id SERIAL PRIMARY KEY,
    my_enum enum('foo', 'bar', 'baz_quxx') NOT NULL
);
```
sqlite -
```sql
CREATE TABLE my_table (
    id SERIAL PRIMARY KEY,
    my_enum TEXT CHECK(my_enum IN ('foo', 'bar', 'baz_quxx')) NOT NULL
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

Postgres arrays work too! See [this example.](tests/src/pg_array.rs)

### Enums Explained

Enums work slightly differently in each of the three databases.
* In Postgres, one declares an enum as a separate type within a schema, which may then be used in multiple tables. Internally, an enum value is encoded as an int (four bytes) and stored inline within a row - a much more efficient representation than a string.
* MySQL is similar except the enum is not declared as a separate type and is 'local' to it's parent table. It is encoded as either one or two bytes.
* sqlite on the other hand does not really have enums - in fact, it does [not really have types](https://dba.stackexchange.com/questions/106364/text-string-stored-in-sqlite-integer-column); you can store any kind of data in any column and it won't complain. Instead we emulate static checking by adding the `CHECK` command, as per above. This does not give a more compact encoding but does ensure data consistency. Note that if you somehow retreive some other invalid text as an enum, `diesel` will error at the point of deserialization.

### Type renaming

Diesel maintains a set of internal types which correspond one-to-one to the types available in various relational databases. Each internal type then maps to some kind of Rust native type. e.g. `diesel::types::Integer` maps to `i32`. So, when we create a new type in Postgres with `CREATE TYPE ...`, we must also create a corresponding type in Diesel, and then create a mapping to some native Rust type (our enum). Hence there are three types we need to be aware of.

By default, the Postgres and Diesel internal types are inferred from the name of the Rust enum. Specifically, we assume `MyEnum` corresponds to `my_enum` in Postgres and `MyEnumMapping` in Diesel. (The Diesel type is created by the plugin, the Postgres type must be created in SQL).

These defaults can be overridden with the attributes `#[PgType = "..."]` and `#[DieselType = "..."]`. (The `PgType` annotation has no effect on `MySQL` or `sqlite`).

Similarly, we assume that the possible ENUM variants are simply the Rust enum variants translated to `snake_case`. These can be renamed with the inline annotation `#[db_rename = "..."]`.

See [this test](tests/src/rename.rs) for an example of renaming.

#### `print-schema` and `infer-schema!`

The `print-schema` command (from `diesel_cli`) attempts to connect to an existing DB and generate a correct mapping of Postgres columns to Diesel internal types. If a custom ENUM exists in the database, Diesel will simply assume that the internal mapping type is the ENUM name, Title-cased (e.g. `my_enum` -> `My_enum`). Therefore the derived mapping name must also be corrected with the `DieselType` attribute e.g. `#[DieselType = "My_enum"]`.

Unfortunately the `infer_schema!` is not compatible with this crate.

### License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
