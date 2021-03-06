use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::models::upload::{Upload, ALL_COLUMNS as ALL_UPLOAD_COLUMNS};
use crate::models::user::User;
use crate::schema::upload_comments;
use crate::schema::uploads;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[belongs_to(User)]
#[belongs_to(Upload)]
#[table_name = "upload_comments"]
pub struct UploadComment {
    pub id: i64,
    pub upload_id: i32,
    pub user_id: i32,
    pub comment: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UploadComment {
    pub fn is_edited(&self) -> bool {
        self.created_at != self.updated_at
    }

    pub fn is_author(&self, user_id: i32) -> bool {
        self.id == user_id as i64
    }
}

#[derive(Debug, Insertable)]
#[table_name = "upload_comments"]
pub struct NewUploadComment {
    pub upload_id: i32,
    pub user_id: i32,
    pub comment: String,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[table_name = "upload_comments"]
pub struct UpdateUploadComment {
    pub comment: String,
}

/// Inserts a new [`UploadComment`] into the database.
pub fn insert(
    conn: &PgConnection,
    upload_comments: &NewUploadComment,
) -> QueryResult<UploadComment> {
    upload_comments
        .insert_into(upload_comments::table)
        .get_result(conn)
}

/// Gets an [`UploadComment`] by a given `comment_id`.
pub fn get_comment_by_id(conn: &PgConnection, comment_id: i64) -> Option<UploadComment> {
    upload_comments::table
        .filter(upload_comments::id.eq(comment_id))
        .first::<UploadComment>(conn)
        .ok()
}

/// Gets all comments + authors by `upload_id`.
pub fn get_by_upload_id(
    conn: &PgConnection,
    upload_id: i32,
) -> QueryResult<Vec<(UploadComment, User)>> {
    upload_comments::table
        .inner_join(users::table)
        .filter(upload_comments::upload_id.eq(upload_id))
        .order(upload_comments::created_at.asc())
        .load::<(UploadComment, User)>(conn)
}

/// Updates a given [`UploadComment`] with new column values.
pub fn update(
    conn: &PgConnection,
    id: i64,
    comment: &UpdateUploadComment,
) -> QueryResult<UploadComment> {
    diesel::update(upload_comments::table.filter(upload_comments::id.eq(id)))
        .set(comment)
        .get_result::<UploadComment>(conn)
}

pub fn get_comment_count_by_user_id(conn: &PgConnection, user_id: i32) -> i64 {
    use diesel::dsl::count;

    upload_comments::table
        .select(count(upload_comments::id))
        .filter(upload_comments::user_id.eq(user_id))
        .first::<i64>(conn)
        .unwrap_or_default()
}

pub fn get_paginated_comments(
    conn: &PgConnection,
    user_id: i32,
    page: i64,
    per_page: i64,
) -> Vec<(UploadComment, Upload)> {
    upload_comments::table
        .inner_join(uploads::table)
        .select((upload_comments::all_columns, ALL_UPLOAD_COLUMNS))
        .filter(upload_comments::user_id.eq(user_id))
        .order_by(upload_comments::created_at.desc())
        .limit(per_page)
        .offset((page - 1) * per_page)
        .load::<(UploadComment, Upload)>(conn)
        .unwrap_or_default()
}

pub struct RecentComment {
    pub comment: UploadComment,
    pub author: User,
    pub upload: Upload,
}

impl From<(UploadComment, User, Upload)> for RecentComment {
    fn from((comment, author, upload): (UploadComment, User, Upload)) -> RecentComment {
        RecentComment {
            comment,
            author,
            upload,
        }
    }
}

/// Gets the N-most recent comments and their user.
pub fn get_recent_comments(conn: &PgConnection) -> Vec<(UploadComment, User, Upload)> {
    upload_comments::table
        .inner_join(users::table)
        .inner_join(uploads::table)
        .select((
            upload_comments::all_columns,
            users::all_columns,
            ALL_UPLOAD_COLUMNS,
        ))
        .order(upload_comments::created_at.desc())
        .limit(10)
        .load::<(UploadComment, User, Upload)>(conn)
        .unwrap_or_default()
}
