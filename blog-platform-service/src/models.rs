use rocket::serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Validate)]
#[serde(crate = "rocket::serde")]
pub struct Blog {
    #[validate(length(min = 1))]
    pub title: String,
    #[validate(length(min = 1))]
    pub content: String,
    #[validate(length(min = 1))]
    pub category: String,
    #[validate(length(min = 1))]
    pub tags: Vec<String>,

    #[serde(skip_deserializing, default)]
    pub id: u32,
    #[serde(skip_deserializing, default, rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_deserializing, default, rename = "updatedAt")]
    pub updated_at: String,
    #[serde(skip_deserializing, skip_serializing, default)]
    pub _deleted_at: Option<String>,
}

impl Blog {
    pub fn on_create(mut self, id: u32) -> Blog {
        let now = match OffsetDateTime::now_utc().format(&Rfc3339).ok() {
            Some(now) => now,
            _ => String::new(),
        };

        self.id = id;
        self.created_at = now.clone();
        self.updated_at = now;
        self
    }

    pub fn on_update(mut self) -> Blog {
        let now = match OffsetDateTime::now_utc().format(&Rfc3339).ok() {
            Some(now) => now,
            _ => String::new(),
        };

        self.updated_at = now;
        self
    }
}
