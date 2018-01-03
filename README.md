# Diesel derive enum

Example: 

```rust
// define your enum

#[derive(PgEnum)]
#[PgType = "my_type"]  // this is the name of the type within the database
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
        my_enum_column -> MyType,
    }
}

// define a struct which populates/queries the table
#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "custom_types"]
struct  MyTable {
    id: i32,
    custom_enum: MyEnum,
}
```

SQL:

```sql
CREATE TYPE my_type AS ENUM ('foo', 'bar', 'baz_quxx');
CREATE TABLE custom_types (
  id SERIAL PRIMARY KEY,
  custom_enum my_type NOT NULL
);
```
