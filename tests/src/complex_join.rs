use diesel::prelude::*;

use crate::common::*;

table! {
    users (id) {
        id -> Integer,
    }
}

#[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
#[diesel(postgres_type(name = "server_status"))]
pub struct Server_status_pg;

#[cfg(feature = "postgres")]
table! {
    use diesel::sql_types::*;
    use super::Server_status_pg;
    servers (id) {
        id -> Integer,
        user_id -> Integer,
        status -> Server_status_pg,
    }
}

#[cfg(not(feature = "postgres"))]
table! {
    use diesel::sql_types::*;
    use super::Server_status;
    servers (id) {
        id -> Integer,
        user_id -> Integer,
        status -> Server_status,
    }
}

joinable!(servers -> users (user_id));
allow_tables_to_appear_in_same_query!(users, servers);

#[derive(diesel_derive_enum::DbEnum, Clone, Debug, PartialEq)]
#[cfg_attr(
    any(feature = "mysql", feature = "sqlite"),
    DieselType = "Server_status"
)]
#[cfg_attr(feature = "postgres", DieselTypePath = "Server_status_pg")]
enum ServerStatus {
    Started,
    Stopped,
}

#[derive(Insertable, Identifiable, Queryable, PartialEq, Debug)]
#[diesel(table_name = users)]
struct User {
    id: i32,
}

#[derive(Insertable, Queryable, Associations, PartialEq, Debug)]
#[diesel(belongs_to(User))]
#[diesel(table_name = servers)]
struct Server {
    id: i32,
    user_id: i32,
    status: ServerStatus,
}

#[cfg(feature = "postgres")]
pub fn create_table(conn: &mut PgConnection) {
    use diesel::connection::SimpleConnection;
    conn.batch_execute(
        r#"
        CREATE TYPE server_status AS ENUM ('started', 'stopped');
        CREATE TABLE users (
            id SERIAL PRIMARY KEY
        );
        CREATE TABLE servers (
            id SERIAL PRIMARY KEY,
            user_id INTEGER REFERENCES users (id),
            status server_status
        );
    "#,
    )
    .unwrap();
}

#[test]
#[cfg(feature = "postgres")]
fn test_complex_join() {
    let conn = &mut get_connection();
    create_table(conn);
    let some_users = vec![User { id: 1 }, User { id: 2 }];
    let some_servers = vec![
        Server {
            id: 1,
            user_id: 1,
            status: ServerStatus::Started,
        },
        Server {
            id: 2,
            user_id: 1,
            status: ServerStatus::Stopped,
        },
        Server {
            id: 3,
            user_id: 2,
            status: ServerStatus::Started,
        },
    ];
    diesel::insert_into(users::table)
        .values(&some_users)
        .execute(conn)
        .unwrap();
    diesel::insert_into(servers::table)
        .values(&some_servers)
        .execute(conn)
        .unwrap();
    let (user, server) = users::table
        .find(1)
        .left_join(
            servers::table.on(servers::dsl::user_id
                .eq(users::dsl::id)
                .and(servers::dsl::status.eq(ServerStatus::Started))),
        )
        .first::<(User, Option<Server>)>(conn)
        .unwrap();
    assert_eq!(user, User { id: 1 });
    assert_eq!(
        server.unwrap(),
        Server {
            id: 1,
            user_id: 1,
            status: ServerStatus::Started
        }
    );
}
