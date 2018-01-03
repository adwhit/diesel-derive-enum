# Diesel derive enum

This crate allows one to derive the Diesel boilerplate necessary to use Rust enums directly with
postgres databases.

It is a fairly literal translation of [this code](https://github.com/diesel-rs/diesel/blob/8f8dd92135a788c7d0f2c5202dcb4d05339a0cc1/diesel_tests/tests/custom_types.rs) from the Diesel test suite.

Example usage: 

```rust
// define your enum

#[derive(PgEnum)]
#[PgType = "my_type"]  // This is the name of the type within the database
                       // A corresponding Rust type "MyType" will be created
pub enum MyEnum {
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
        my_enum -> MyType,
    }
}

// define a struct which populates/queries the table
#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "my_table"]
struct  MyTable {
    id: i32,
    my_enum: MyEnum,
}
```

SQL to create corresponding table:

```sql
CREATE TYPE my_type AS ENUM ('foo', 'bar', 'baz_quxx');
CREATE TABLE my_table (
  id SERIAL PRIMARY KEY,
  my_enum my_type NOT NULL
);
```

Now we can insert and retrieve MyEnum directly:

```rust
let data = vec![
    MyTable {
        id: 1,
        my_enum: MyEnum::Foo,
    },
    MyTable {
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
