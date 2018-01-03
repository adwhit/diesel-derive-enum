#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;

#[derive(Debug, PartialEq, PgEnum)]
#[PgType = "MyType"]
pub enum MyEnum {
    Foo,
    Bar,
    Baz,
}

table! {
    use diesel::types::Integer;
    use super::MyType;
    custom_types {
        id -> Integer,
        custom_enum -> MyType,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "custom_types"]
struct HasCustomTypes {
    id: i32,
    custom_enum: MyEnum,
}

#[test]
fn custom_types_round_trip() {
    let data = vec![
        HasCustomTypes {
            id: 1,
            custom_enum: MyEnum::Foo,
        },
        HasCustomTypes {
            id: 2,
            custom_enum: MyEnum::Bar,
        },
    ];
    // let connection = connection();
    // connection
    //     .batch_execute(
    //         r#"
    //     CREATE TYPE my_type AS ENUM ('foo', 'bar');
    //     CREATE TABLE custom_types (
    //         id SERIAL PRIMARY KEY,
    //         custom_enum my_type NOT NULL
    //     );
    // "#,
    //     )
    //     .unwrap();

    // let inserted = insert_into(custom_types::table)
    //     .values(&data)
    //     .get_results(&connection)
    //     .unwrap();
    // assert_eq!(data, inserted);
}
