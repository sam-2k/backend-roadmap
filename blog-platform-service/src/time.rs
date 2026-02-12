use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub fn get_now() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).unwrap()
}
