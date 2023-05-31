#[cfg(not(feature = "custom"))]
pub mod schema;
#[cfg(not(feature = "custom"))]
pub mod with_default_schema;

#[cfg(feature = "custom")]
pub mod custom_schema;

#[cfg(feature = "custom")]
pub mod with_custom_schema;

pub use diesel::pg::PgConnection as Conn;
pub use diesel::Connection;

pub fn get_connection() -> Conn {
    let database_url = ::std::env::var("DATABASE_URL").expect("Env var DATABASE_URL not set");
    let conn =
        Conn::establish(&database_url).expect(&format!("Failed to connect to {}", database_url));
    conn
}

#[cfg(test)]
mod tests {

    #[cfg(not(feature = "custom"))]
    use crate::schema::simple;
    #[cfg(not(feature = "custom"))]
    use crate::with_default_schema::*;

    #[cfg(feature = "custom")]
    use crate::custom_schema::simple;
    #[cfg(feature = "custom")]
    use crate::with_custom_schema::*;

    use diesel::prelude::*;

    #[test]
    fn round_trip() {
        let mut conn = crate::get_connection();
        let this = Simple {
            id: 1,
            some_value: MyEnum::Foo,
        };
        let that = insert(&mut conn, &this).unwrap();
        assert_eq!(this, that);

        // make a query that requires QueryId trait to exist
        let _: Vec<Simple> = simple::table
            .filter(simple::some_value.eq(MyEnum::Foo))
            .limit(1)
            .load(&mut conn)
            .unwrap();
    }
}
