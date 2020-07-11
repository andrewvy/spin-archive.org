use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::models::user::User;
use crate::schema::posts;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, QueryableByName, Clone)]
#[table_name = "posts"]
pub struct Post {
    pub id: i64,
    pub thread_id: i64,
    pub author_id: i32,
    pub content: String,
    pub is_deleted: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub content: &'a str,
    pub thread_id: i64,
    pub author_id: i32,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[table_name = "posts"]
pub struct UpdatePost<'a> {
    pub content: &'a str,
}

/// Gets a [`Post`] by a given `id`.
pub fn by_id(conn: &PgConnection, post_id: i64) -> Option<Post> {
    posts::table
        .filter(posts::id.eq(post_id))
        .first::<Post>(conn)
        .ok()
}

/// Gets posts in default order, by thread_id
pub fn by_thread_id(conn: &PgConnection, thread_id: i64) -> Vec<(Post, User)> {
    posts::table
        .filter(posts::thread_id.eq(thread_id))
        .filter(posts::is_deleted.eq(false))
        .inner_join(users::table)
        .select((posts::all_columns, users::all_columns))
        .order(posts::created_at.asc())
        .load::<(Post, User)>(conn)
        .unwrap_or_default()
}

pub fn insert(conn: &PgConnection, post: &NewPost) -> QueryResult<Post> {
    post.insert_into(posts::table)
        .returning(posts::all_columns)
        .get_result(conn)
}

/// Updates a given [`Post`] with new column values.
pub fn update(conn: &PgConnection, id: i64, post: &UpdatePost) -> QueryResult<Post> {
    diesel::update(posts::table.filter(posts::id.eq(id)))
        .set(post)
        .get_result::<Post>(conn)
}