use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::PgConnection;
use log::{debug, warn};
use nanoid::nanoid;

use crate::models::audit_log::{self, AuditLog};
use crate::models::upload::{self, PendingUpload, UpdateUpload, Upload, UploadStatus};
use crate::models::user::User;
use crate::schema::upload_views;
use crate::services::{audit_service, encoder_service, tag_service};

#[derive(Insertable)]
#[table_name = "upload_views"]
pub struct View {
  pub upload_id: i32,
}

#[allow(dead_code)]
pub(crate) enum UploadError {
  AlreadyExists,
  DatabaseError,
  NotFound,
}

/// Creates a new pending upload.
pub(crate) fn new_pending_upload(
  conn: &PgConnection,
  user: &User,
  file_name: &str,
  file_ext: &str,
) -> Result<Upload, UploadError> {
  let pending_upload = PendingUpload {
    status: UploadStatus::Pending,
    file_id: nanoid!(),
    video_encoding_key: nanoid!(),
    uploader_user_id: user.id,
    file_name: file_name.to_owned(),
    file_ext: file_ext.to_owned(),
  };

  upload::insert_pending_upload(&conn, &pending_upload).map_err(|_| UploadError::DatabaseError)
}

/// Finalizes a pending upload, which means the user has finished uploading the file and
/// we can move the upload for later processing.
pub(crate) fn finalize_upload(
  conn: &PgConnection,
  file_id: &str,
  tags: &str,
  source: &str,
  description: &str,
  original_upload_date: Option<NaiveDate>,
) -> Result<Upload, UploadError> {
  match upload::get_by_file_id(&conn, &file_id) {
    Some(
      upload @ Upload {
        status: UploadStatus::Pending,
        ..
      },
    ) => {
      let update_upload = UpdateUpload {
        id: upload.id,
        status: UploadStatus::Processing,
        tag_string: sanitize_tags(tags),
        source: Some(source.to_owned()),
        description: description.to_string(),
        original_upload_date,
      };

      match upload::update(&conn, &update_upload) {
        Ok(upload) => {
          after_edit_hooks(&conn, &upload);

          match encoder_service::enqueue_upload(&upload) {
            Ok(_job) => {
              debug!("[encoding] Started job id {}", upload.video_encoding_key);
            }
            Err(e) => {
              warn!(
                "[encoding] Job error: {:?} for job id {}",
                e, upload.video_encoding_key
              );
            }
          }

          Ok(upload)
        }
        Err(_err) => Err(UploadError::DatabaseError),
      }
    }
    Some(_upload) => Err(UploadError::AlreadyExists),
    None => Err(UploadError::NotFound),
  }
}

/// Updates an already published upload.
pub(crate) fn update_upload(
  conn: &PgConnection,
  user_id: i32,
  file_id: &str,
  tags: &str,
  source: &str,
  description: &str,
  original_upload_date: Option<NaiveDate>,
) -> Result<Upload, UploadError> {
  match upload::get_by_file_id(&conn, &file_id) {
    Some(upload) => {
      let new_tag_string = sanitize_tags(tags);

      let update_upload = UpdateUpload {
        id: upload.id,
        status: upload.status,
        tag_string: new_tag_string.clone(),
        source: Some(source.to_owned()),
        description: description.to_string(),
        original_upload_date,
      };

      audit_service::create_audit_log(
        &conn,
        "uploads",
        "tag_string",
        upload.id,
        user_id,
        &upload.tag_string,
        &new_tag_string,
      );

      audit_service::create_audit_log(
        &conn,
        "uploads",
        "source",
        upload.id,
        user_id,
        &upload.source.unwrap_or("".to_string()),
        &source,
      );

      audit_service::create_audit_log(
        &conn,
        "uploads",
        "description",
        upload.id,
        user_id,
        &upload.description,
        &description,
      );

      match upload::update(&conn, &update_upload) {
        Ok(upload) => {
          after_edit_hooks(&conn, &upload);
          Ok(upload)
        }
        Err(_err) => Err(UploadError::DatabaseError),
      }
    }
    None => Err(UploadError::NotFound),
  }
}

pub fn after_edit_hooks(conn: &PgConnection, upload: &Upload) {
  let _ = tag_service::create_from_tag_string(&conn, &upload.tag_string);
}

pub fn sanitize_tags<'a>(tags: &'a str) -> String {
  tags
    .split_whitespace()
    .map(|str| str.to_lowercase())
    .filter(|str| str.len() <= 60)
    .collect::<Vec<_>>()
    .join(" ")
}

/// Increments the view count for an upload.
pub fn increment_view_count(conn: &PgConnection, upload_id: i32) {
  let view = View { upload_id };

  let _ = view.insert_into(upload_views::table).execute(conn);
}

/// Gets the view count for an upload.
pub fn get_view_count(conn: &PgConnection, upload_id: i32) -> i64 {
  use diesel::prelude::*;

  upload_views::table
    .select(diesel::dsl::count_star())
    .filter(upload_views::upload_id.eq(upload_id))
    .first(conn)
    .unwrap_or(0)
}

/// Gets the associated uploader user.
pub fn get_uploader_user(conn: &PgConnection, upload: &Upload) -> User {
  use crate::models::user;

  user::get_user_by_id(&conn, upload.uploader_user_id.expect("No uploader user")).unwrap()
}

/// Gets an audit log for a particular upload.
pub fn get_audit_log(conn: &PgConnection, upload: &Upload) -> Vec<(AuditLog, User)> {
  audit_log::get_by_row_id(conn, "uploads", upload.id).unwrap_or_default()
}
