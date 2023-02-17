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
