use std::collections::HashSet;

use diesel::PgConnection;
use log::debug;

use crate::models::tag::{self, NewTag, Tag};
use crate::models::upload::{Upload, UploadStatus};
use crate::schema::uploads;

pub fn create_from_tag_string(conn: &PgConnection, tag_string: &str) {
  let tags = sanitize_tags(tag_string);

  for tag_name in tags.iter() {
    let new_tag = NewTag {
      name: tag_name.to_owned(),
    };

    let _ = tag::insert(&conn, &new_tag);
  }
}

pub fn sanitize_tags<'a>(tags: &'a str) -> Vec<String> {
  tags
    .split_whitespace()
    .map(|str| str.to_lowercase())
    .collect::<Vec<_>>()
}

pub fn rebuild(conn: &PgConnection) {
  use diesel::prelude::*;

  let limit = 250;
  let mut offset: i64 = 0;

  loop {
    let mut buffer: HashSet<String> = HashSet::new();

    let tag_strings: Vec<String> = uploads::table
      .select(uploads::tag_string)
      .filter(uploads::status.eq(UploadStatus::Completed))
      .order(uploads::id)
      .limit(limit)
      .offset(offset)
      .load::<String>(conn)
      .unwrap();

    let record_count = tag_strings.len() as i64;

    dedupe_tags(&tag_strings, &mut buffer);

    for tag_name in buffer.iter() {
      let new_tag = NewTag {
        name: tag_name.to_owned(),
      };

      let _ = tag::insert(&conn, &new_tag);
    }

    offset += record_count;

    debug!("[tag_service] rebuild limit={}, offset={}", limit, offset);

    if record_count < limit {
      debug!("[tag_service] rebuild finished!");
      break;
    }
  }
}

fn dedupe_tags<'a>(tag_strings: &Vec<String>, buffer: &'a mut HashSet<String>) {
  for tag_string in tag_strings.iter() {
    let sanitized_tags = sanitize_tags(tag_string);

    for tag in sanitized_tags {
      buffer.insert(tag);
    }
  }
}