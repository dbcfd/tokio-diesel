#[macro_use]
extern crate diesel;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    sql_query,
};
use std::error::Error;
use tokio_diesel::*;
use uuid::Uuid;

// Schema
table! {
    users (id) {
        id -> Uuid,
    }
}

#[derive(Queryable)]
pub struct User {
    id: uuid::Uuid,
}

#[tokio::test(flavor = "multi_thread")]
async fn test_db_ops() -> Result<(), Box<dyn Error>> {
    let addr = std::env::var("DB_ADDR")
        .unwrap_or_else(|_| "postgres://postgres@localhost".to_string());
    let manager = ConnectionManager::<PgConnection>::new(&addr);
    let pool = Pool::builder().build(manager)?;

    let _ = sql_query(include_str!("./create_users.sql"))
        .execute_async(&pool)
        .await;

    // Add
    println!("add a user");
    let user_id = Uuid::new_v4();
    diesel::insert_into(users::table)
        .values(users::id.eq(user_id.clone()))
        .execute_async(&pool)
        .await?;

    // Count
    let num_users: i64 = users::table.count().get_result_async(&pool).await?;
    println!("now there are {:?} users", num_users);

    assert!(num_users > 0);

    // Boxed query
    let boxed_user_query: diesel::query_builder::BoxedSelectStatement<'_, _, _, _> =
        users::table.filter(users::id.eq(user_id))
            .into_boxed();
    let user: User = boxed_user_query.get_result_async(&pool).await?;

    assert_eq!(user.id, user_id);


    Ok(())
}
