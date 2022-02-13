# diesel-derive-enum
[![crates.io](https://img.shields.io/crates/v/diesel-derive-enum.svg)](https://crates.io/crates/diesel-derive-enum)
![Build Status](https://github.com/adwhit/diesel-derive-enum/workflows/CI/badge.svg)

Use Rust enums directly with [`diesel`](https://github.com/diesel-rs/diesel) ORM.

The latest release, `2.0.0-rc.0`, is tested against `diesel 2.0.0-rc.0` and `rustc 1.51.0`.

*Note:* The master branch of this repository tracks `diesel` master, and will **not** work with `diesel 1.x`.
In addition, the API has changed somewhat - if you are using `diesel 1.x` please refer to
the [README in the diesel-1 branch](https://github.com/adwhit/diesel-derive-enum/blob/diesel-1/README.md).
What follows will _only work for `diesel 2.0`.

## Setup

### Postgres

Cargo.toml:
```toml
[dependencies]
diesel-derive-enum = { version = "2.0.0-rc.0", features = ["postgres"] }
```

As of `diesel-2.0.0-rc`, we recommend using [diesel-cli](http://diesel.rs/guides/configuring-diesel-cli.html)
to generate correct SQL <-> Rust bindings.

Suppose your project has the following `diesel.toml`:

``` toml
[print_schema]
file = "src/schema.rs"
```

And the following SQL:
```sql
CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');

CREATE TABLE my_table (
  id SERIAL PRIMARY KEY,
  some_enum my_enum NOT NULL
);
```

Then diesel-cli will generate something like the following:

```rust
// src/schema.rs

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "my_enum"))]
    pub struct MyEnum;
}

table! {
    use diesel::types::Integer;
    use super::sql_types::MyEnum;

    my_table {
        id -> Integer,
        some_enum -> MyEnum
    }
}
```
Now we can use `diesel-derive-enum` to hook in our own enum:

```rust
// src/my_code.rs

#[derive(diesel_derive_enum::DbEnum)]
#[DieselTypePath = "crate::schema::sql_types::MyEnum"]
pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}
```

### MySQL

Cargo.toml:
```toml
[dependencies]
diesel-derive-enum = { version = "2.0.0-rc.0", features = ["mysql"] }
```

SQL:
```sql
CREATE TABLE my_table (
    id SERIAL PRIMARY KEY,
    my_enum enum('foo', 'bar', 'baz_quxx') NOT NULL  -- note: snake_case
);
```

Rust:
```rust
#[derive(diesel_derive_enum::DbEnum)]
pub enum MyEnum {
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
```

### sqlite


Cargo.toml:
```toml
[dependencies]
diesel-derive-enum = { version = "2.0.0-rc.0", features = ["sqlite"] }
```

SQL:
```sql
CREATE TABLE my_table (
    id SERIAL PRIMARY KEY,
    my_enum TEXT CHECK(my_enum IN ('foo', 'bar', 'baz_quxx')) NOT NULL   -- note: snake_case
);
```

Rust:
``` rust
#[derive(diesel_derive_enum::DbEnum)]
pub enum MyEnum {
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
```

## Usage

Usage is similar regardless of your chosen database.
We can define a struct with which to populate/query the table:

``` rust
#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = my_table)]
struct  MyRow {
    id: i32,
    some_enum: MyEnum,
}
```

And use it in the natural way:

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

### Enums Representations

Enums are not part of the SQL standard and have database-specific implementations.

* In Postgres, we declare an enum as a separate type within a schema (`CREATE TYPE ...`),
  which may then be used in multiple tables. Internally, an enum value is encoded as an int (four bytes)
  and stored inline within a row (a much more efficient representation than a string).

* MySQL is similar except the enum is not declared as a separate type and is 'local' to
  it's parent table. It is encoded as either one or two bytes.

* sqlite does not have enums - in fact, it does
  [not really have types](https://dba.stackexchange.com/questions/106364/text-string-stored-in-sqlite-integer-column);
  you can store any kind of data in any column. Instead we emulate static checking by
  adding the `CHECK` command, as per above. This does not give a more compact encoding
  but does ensure better data integrity. Note that if you somehow retreive some other invalid
  text as an enum, `diesel` will error at the point of deserialization.

### How It Works

Diesel maintains a set of internal types which correspond one-to-one to the types available in various
relational databases. Each internal type in turn maps to some kind of Rust native type.
e.g. Postgres `INTEGER` maps to `diesel::types::Integer` maps to `i32`.

*For `postgres` only*, as of `diesel-2.0.0-rc.0`, diesel will create the 'dummy' internal enum type as part
of the schema generation process. This crate will attempt to locate this dummy type at the
the default path of `crate::schema::sql_types::{enum_name}`. This location can be overridden with the
`DieselTypePath` attribute.

For `mysql` and `sqlite`, the intenal type is *not* automatically generated, so this macro will instead create it
with the default name `{enum_name}Mapping`. This name can be overridden with the `DieselType` attribute.

In either case, this macro will then implement various traits on the internal type.
This macro will also implement various traits on the user-defined `enum` type.
The net result of this is that the user-defined enum can be directly inserted into (and retrieved
from) the diesel database.

Note that by default we assume that the possible SQL ENUM variants are simply the Rust enum variants
translated to `snake_case`.  These can be renamed with the inline annotation `#[db_rename = "..."]`.

See [this test](tests/src/rename.rs) for an example of renaming.

You can override the `snake_case` assumption for the entire enum using the `#[DbValueStyle = "..."]`
attribute.  Individual variants can still be renamed using `#[db_rename = "..."]`.

| DbValueStyle   | Variant | Value   |
|:-------------------:|:---------:|:---|
| camelCase | BazQuxx | "bazQuxx" |
| kebab-case | BazQuxx | "baz-quxx" |
| PascalCase | BazQuxx | "BazQuxx" |
| SCREAMING_SNAKE_CASE | BazQuxx | "BAZ_QUXX" |
| SCREAMINGCONCAT | BazQuxx | "BAZQUXX" |
| snake_case | BazQuxx | "baz_quxx" |
| verbatim | Baz__quxx | "Baz__quxx" |

See [this test](tests/src/value_style.rs) for an example of changing the output style.

### License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
