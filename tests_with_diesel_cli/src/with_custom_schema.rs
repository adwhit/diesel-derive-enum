use diesel::pg::PgConnection as Conn;
use diesel::prelude::*;
use diesel::result::Error;

use crate::custom_schema::simple;

#[derive(diesel_derive_enum::DbEnum, Debug, Copy, Clone, PartialEq, Eq)]
// NOTE: no ExistingTypePath, so we generate the mapping type ourselves
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut conn = crate::get_connection();
        let this = Simple {
            id: 2,
            some_value: MyEnum::BazQuxx,
        };
        let that = insert(&mut conn, &this).unwrap();
        assert_eq!(this, that);
    }
}

pub(crate) mod export {
    pub use super::MyEnumMapping as MyEnum;
}
