# diesel-derive-enum
[![crates.io](https://img.shields.io/crates/v/diesel-derive-enum.svg)](https://crates.io/crates/diesel-derive-enum)
![Build Status](https://github.com/adwhit/diesel-derive-enum/workflows/CI/badge.svg)

Use Rust enums directly with [`diesel`](https://github.com/diesel-rs/diesel) ORM.

Diesel is great, but wouldn't it be nice if this would work?

``` rust

use crate::schema::my_table;

pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}

fn do_some_work(data: MyEnum, connection: &mut Connection) {
    insert_into(my_table)
        .values(&data)
        .execute(connection)
        .unwrap();
}
```

Unfortunately, it won't work out of the box, because any type which
we wish to use with Diesel must implement various traits.
Tedious to do by hand, easy to do with a `derive` macro - enter `diesel-derive-enum`.

The latest release, `3.0.0-beta.1`, is tested against `diesel 2.2.8` and `rustc 1.82`
For earlier versions of `diesel`, check out the `2.1.0` and earlier releases of this crate.

## What's New in v3

Version 3.0.0-beta.1 introduces several significant **BREAKING CHANGES**:

1. **Attribute Namespacing**: All attributes are now namespaced under `db_enum(...)` for better clarity and organization
2. **Clone Implementation Change**: Clone is no longer implemented by default on SQL types (now opt-in rather than opt-out)
3. **MSRV Update**: The minimum supported Rust version is now 1.82

## Setup with Diesel CLI

This crate integrates nicely with
[diesel-cli](http://diesel.rs/guides/configuring-diesel-cli.html) -
this is the recommended workflow.
Note that for now, this **only** works with Postgres - for other databases,
or if not using Diesel CLI, see the next section.

Cargo.toml:
```toml
[dependencies]
diesel-derive-enum = { version = "3.0.0-beta.1", features = ["postgres"] }
```

Suppose our project has the following `diesel.toml` (as generated by `diesel setup`):

``` toml
[print_schema]
file = "src/schema.rs"
custom_type_derives = ["diesel::query_builder::QueryId", "Clone"]
```

And the following SQL:
```sql
CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');

CREATE TABLE my_table (
  id SERIAL PRIMARY KEY,
  some_enum my_enum NOT NULL
);
```

Then running `$ diesel migration run` will generate code something like:

```rust
// src/schema.rs -- autogenerated

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId, Clone)]
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
#[db_enum(existing_type_path = "crate::schema::sql_types::MyEnum")]
pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}
```

Note the `db_enum(existing_type_path = "...")` attribute. This instructs this crate to import the
(remote, autogenerated) type and implement various traits upon it. That's it!
Now we can use `MyEnum` with `diesel` (see 'Usage' below).


## Setup without Diesel CLI

If you are using `mysql` or `sqlite`, or you aren't using `diesel-cli` to generate 
your schema, the setup is a little different.

### Postgres

Cargo.toml:
```toml
[dependencies]
diesel-derive-enum = { version = "3.0.0-beta.1", features = ["postgres"] }
```

SQL:
```sql
CREATE TYPE my_enum AS ENUM ('foo', 'bar', 'baz_quxx');

CREATE TABLE my_table (
  id SERIAL PRIMARY KEY,
  some_enum my_enum NOT NULL
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

### MySQL

Cargo.toml:
```toml
[dependencies]
diesel-derive-enum = { version = "3.0.0-beta.1", features = ["mysql"] }
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
diesel-derive-enum = { version = "3.0.0-beta.1", features = ["sqlite"] }
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

Once set up, usage is similar regardless of your chosen database.
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

## Attribute Reference

### Type attributes

| Attribute | Description | Default | Example |
|-----------|-------------|---------|---------|
| `existing_type_path` | Path to corresponding Diesel type | None | `#[db_enum(existing_type_path = "crate::schema::sql_types::MyEnum")]` |
| `diesel_type` | Name for the Diesel type to create | `<enum name>Mapping` | `#[db_enum(diesel_type = "CustomMapping")]` |
| `pg_type` | Name of PostgreSQL type | `<enum name in snake_case>` | `#[db_enum(pg_type = "custom_type")]` |
| `value_style` | Renaming style from Rust enum to database | `snake_case` | `#[db_enum(value_style = "camelCase")]` |
| `impl_clone_on_sql_mapping` | Implement Clone for the SQL type | `false` | `#[db_enum(impl_clone_on_sql_mapping)]` |

### Variant attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `rename` | Specify database name for a variant | `#[db_enum(rename = "custom_name")]` |

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
  but does ensure better data integrity. Note that if you somehow retrieve some other invalid
  text as an enum, `diesel` will error at the point of deserialization.

### How It Works

Diesel maintains a set of internal types which correspond one-to-one to the types available in various
relational databases. Each internal type in turn maps to some kind of Rust native type.
e.g. Postgres `INTEGER` maps to `diesel::types::Integer` maps to `i32`.

*For `postgres` only*, as of `diesel-2.0.0`, diesel-cli will create the 'dummy' internal
enum mapping type as part of the schema generation process.
We then specify the location of this type with the `existing_type_path` attribute.

In the case where `existing_type_path` is **not** specified, we assume the internal type
has *not* already been generated, so this macro will instead create it
with the default name `{enum_name}Mapping`. This name can be overridden with the `diesel_type` attribute.

In either case, this macro will then implement various traits on the internal type.
This macro will also implement various traits on the user-defined `enum` type.
The net result of this is that the user-defined enum can be directly inserted into (and retrieved
from) the diesel database.

Note that by default we assume that the possible SQL ENUM variants are simply the Rust enum variants
translated to `snake_case`.  These can be renamed with the inline annotation `#[db_enum(rename = "...")]`.

See [tests/src/pg_remote_type.rs](tests/src/pg_remote_type.rs) for an example of using the `existing_type_path` attribute.

You can override the `snake_case` assumption for the entire enum using the `#[db_enum(value_style = "...")]`
attribute.  Individual variants can still be renamed using `#[db_enum(rename = "...")]`.

| Value Style | Variant | Value |
|:-------------------:|:---------:|:---|
| camelCase | BazQuxx | "bazQuxx" |
| kebab-case | BazQuxx | "baz-quxx" |
| PascalCase | BazQuxx | "BazQuxx" |
| SCREAMING_SNAKE_CASE | BazQuxx | "BAZ_QUXX" |
| UPPERCASE | BazQuxx | "BAZQUXX" |
| snake_case | BazQuxx | "baz_quxx" |
| verbatim | Baz__quxx | "Baz__quxx" |

See [tests/src/value_style.rs](tests/src/value_style.rs) for an example of changing the output style.

### License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
