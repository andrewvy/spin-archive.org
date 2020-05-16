use diesel::PgConnection;
use nanoid::nanoid;

use crate::models::upload::{self, PendingUpload, UpdateUpload, Upload, UploadStatus};
use crate::models::user::User;

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
        status: UploadStatus::Completed,
        tag_string: sanitize_tags(tags),
        source: Some(source.to_owned()),
      };

      match upload::update(&conn, &update_upload) {
        Ok(upload) => Ok(upload),
        Err(_err) => Err(UploadError::DatabaseError),
      }
    }
    Some(_upload) => Err(UploadError::AlreadyExists),
    None => Err(UploadError::NotFound),
  }
}

pub fn sanitize_tags<'a>(tags: &'a str) -> String {
  tags
    .split_whitespace()
    .map(|string| string.to_ascii_lowercase())
    .collect::<Vec<_>>()
    .join(" ")
}