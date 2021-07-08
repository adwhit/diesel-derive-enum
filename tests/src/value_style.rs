use diesel::prelude::*;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::common::get_connection;

#[derive(Debug, PartialEq, diesel_derive_enum::DbEnum)]
#[DieselType = "Stylized_Internal_Type"]
#[DieselExistingType = "Stylized_Internal_Type_Pg"]
#[DbValueStyle = "PascalCase"]
pub enum StylizedEnum {
    FirstVariant,
    secondThing,
    third_item,
    FOURTH_VALUE,
    #[db_rename = "crazy fifth"]
    cRaZy_FiFtH,
}

#[cfg(feature = "postgres")]
#[derive(diesel::sql_types::SqlType)]
#[postgres(type_name = "Stylized_External_Type")]
pub struct Stylized_Internal_Type_Pg;
#[cfg(feature = "postgres")]
table! {
    use diesel::sql_types::Integer;
    use super::Stylized_Internal_Type_Pg;
    test_value_style {
        id -> Integer,
        value -> Stylized_Internal_Type_Pg,
    }
}
#[cfg(not(feature = "postgres"))]
table! {
    use diesel::sql_types::Integer;
    use super::Stylized_Internal_Type;
    test_value_style {
        id -> Integer,
        value -> Stylized_Internal_Type,
    }
}

#[derive(Insertable, Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "test_value_style"]
struct TestStylized {
    id: i32,
    value: StylizedEnum,
}

fn sample_data() -> Vec<TestStylized> {
    vec![
        TestStylized {
            id: 1,
            value: StylizedEnum::FirstVariant,
        },
        TestStylized {
            id: 2,
            value: StylizedEnum::secondThing,
        },
        TestStylized {
            id: 3,
            value: StylizedEnum::third_item,
        },
        TestStylized {
            id: 4,
            value: StylizedEnum::FOURTH_VALUE,
        },
        TestStylized {
            id: 5,
            value: StylizedEnum::cRaZy_FiFtH,
        },
    ]
}

#[test]
#[cfg(feature = "postgres")]
fn stylized_round_trip() {
    use diesel::connection::SimpleConnection;
    use diesel::insert_into;
    let data = sample_data();
    let connection = get_connection();
    connection
        .batch_execute(
            r#"
        CREATE TYPE "Stylized_External_Type" AS ENUM (
            'FirstVariant', 'SecondThing', 'ThirdItem', 'FourthValue', 'crazy fifth');
        CREATE TABLE test_value_style (
            id SERIAL PRIMARY KEY,
            value "Stylized_External_Type" NOT NULL
        );
    "#,
        )
        .unwrap();
    let inserted = insert_into(test_value_style::table)
        .values(&data)
        .get_results(&connection)
        .unwrap();
    assert_eq!(data, inserted);
}

#[test]
#[cfg(feature = "mysql")]
fn stylized_round_trip() {
    use diesel::connection::SimpleConnection;
    use diesel::insert_into;
    let data = sample_data();
    let connection = get_connection();
    connection
        .batch_execute(
            r#"
        CREATE TEMPORARY TABLE IF NOT EXISTS test_value_style (
            id SERIAL PRIMARY KEY,
            value enum('FirstVariant', 'SecondThing', 'ThirdItem', 'FourthValue', 'crazy fifth')
                NOT NULL
        );
    "#,
        )
        .unwrap();
    insert_into(test_value_style::table)
        .values(&data)
        .execute(&connection)
        .unwrap();
    let inserted = test_value_style::table
        .load::<TestStylized>(&connection)
        .unwrap();
    assert_eq!(data, inserted);
}

#[test]
#[cfg(feature = "sqlite")]
fn stylized_round_trip() {
    use diesel::connection::SimpleConnection;
    use diesel::insert_into;
    let data = sample_data();
    let connection = get_connection();
    connection
        .batch_execute(
            r#"
        CREATE TABLE test_value_style (
            id SERIAL PRIMARY KEY,
            value TEXT CHECK(value IN (
                'FirstVariant', 'SecondThing', 'ThirdItem', 'FourthValue', 'crazy fifth'
            )) NOT NULL
        );
    "#,
        )
        .unwrap();
    insert_into(test_value_style::table)
        .values(&data)
        .execute(&connection)
        .unwrap();
    let inserted = test_value_style::table
        .load::<TestStylized>(&connection)
        .unwrap();
    assert_eq!(data, inserted);
}
