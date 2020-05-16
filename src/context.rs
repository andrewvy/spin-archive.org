use std::collections::HashMap;

use rocket::request::FlashMessage;
use rocket_contrib::templates::tera::{
  Context as TeraContext, Result as TeraResult, Value as TeraValue,
};

use crate::models::upload::Upload;
use crate::models::user::User;

type GlobalFn = Box<dyn Fn(HashMap<String, TeraValue>) -> TeraResult<TeraValue> + Sync + Send>;

pub(crate) fn flash_context(context: &mut TeraContext, flash: Option<FlashMessage>) {
  if let Some(msg) = flash {
    context.insert("flash_name", msg.name());
    context.insert("flash_message", msg.msg());
  }
}

pub(crate) fn user_context(context: &mut TeraContext, user: Option<&User>) {
  if let Some(user) = user {
    context.insert("user_id", &user.id.to_string());
    context.insert("user_role", &user.role.to_string());
    context.insert("user_can_upload", &user.can_upload());
    context.insert("username", &user.username.clone());
  }
}

/// Template function that returns information about the current release build.
pub fn get_thumbnail_url() -> GlobalFn {
  Box::new(move |args| -> TeraResult<TeraValue> {
    match args.get("upload") {
      Some(value) => match serde_json::from_value::<Upload>(value.clone()) {
        Ok(upload) => Ok(serde_json::to_value(upload.get_thumbnail_url()).unwrap()),
        Err(_) => Err("Could not get upload".into()),
      },
      None => Err("Could not get upload".into()),
    }
  })
}

pub fn get_file_url() -> GlobalFn {
  Box::new(move |args| -> TeraResult<TeraValue> {
    match args.get("upload") {
      Some(value) => match serde_json::from_value::<Upload>(value.clone()) {
        Ok(upload) => Ok(serde_json::to_value(upload.get_file_url()).unwrap()),
        Err(_) => Err("Could not get upload".into()),
      },
      None => Err("Could not get upload".into()),
    }
  })
}

pub fn is_video() -> GlobalFn {
  Box::new(move |args| -> TeraResult<TeraValue> {
    match args.get("upload") {
      Some(value) => match serde_json::from_value::<Upload>(value.clone()) {
        Ok(upload) => Ok(serde_json::to_value(upload.is_video()).unwrap()),
        Err(_) => Err("Could not get upload".into()),
      },
      None => Err("Could not get upload".into()),
    }
  })
}

pub fn split_tags() -> GlobalFn {
  Box::new(move |args| -> TeraResult<TeraValue> {
    match args.get("tags") {
      Some(value) => match serde_json::from_value::<String>(value.clone()) {
        Ok(tag_string) => {
          let tags: Vec<&str> = tag_string.split_whitespace().collect();
          Ok(serde_json::to_value(tags).unwrap())
        }
        Err(_) => Err("Could not get tags".into()),
      },
      None => Err("Could not get upload".into()),
    }
  })
}

pub fn tag_url(value: TeraValue, _args: HashMap<String, TeraValue>) -> TeraResult<TeraValue> {
  match serde_json::from_value::<String>(value.clone()) {
    Ok(tag) => {
      let url = format!("/?q={}", tag);
      Ok(serde_json::to_value(url).unwrap())
    }
    Err(_) => Err("Could not get tags".into()),
  }
}
