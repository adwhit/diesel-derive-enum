use diesel::pg::PgConnection as Conn;
use diesel::prelude::*;
use diesel::result::Error;

use crate::schema::simple;

#[derive(diesel_derive_enum::DbEnum, Debug, Copy, Clone, PartialEq, Eq)]
#[db_enum(existing_type_path = "crate::schema::sql_types::MyEnum")]
pub enum MyEnum {
    Foo,
    Bar,
    BazQuxx,
}

#[derive(Insertable, Queryable, Identifiable, Debug, Clone, PartialEq)]
#[diesel(table_name = simple)]
pub struct Simple {
    pub id: i32,
    pub some_value: MyEnum,
}

pub fn insert(conn: &mut Conn, value: &Simple) -> Result<Simple, Error> {
    diesel::insert_into(simple::table)
        .values(value)
        .get_result(conn)
}
