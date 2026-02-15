#[allow(non_snake_case)]
#[macro_use]
extern crate rocket;

use std::sync::atomic::AtomicU32;

use crate::db::DbConnection;
use rocket::{
    State,
    http::Status,
    serde::json::{Json, Value, to_value},
};
use serde_json::{Map, json};

mod db;
mod models;
mod validate;

type HandlerResult = Result<(Status, Json<Value>), (Status, Json<Value>)>;
struct BlogCount(AtomicU32);

#[post("/", format = "json", data = "<blog_json>")]
fn create(
    blog_json: validate::Validated<models::Blog>,
    blog_count: &State<BlogCount>,
    db_conn: &State<DbConnection>,
) -> (Status, Json<Value>) {
    let blog = blog_json.0.on_create(
        blog_count
            .0
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
    );

    match db_conn.0.lock().expect("Lock is poisoned").execute(
        "INSERT INTO blogs (title, content, category) VALUES (?1, ?2, ?3)",
        (&blog.title, &blog.content, &blog.category),
    ) {
        Ok(_) => match to_value(blog) {
            Ok(value) => (Status::Ok, Json(value)),
            Err(err) => (Status::BadRequest, Json(Value::String(err.to_string()))),
        },
        Err(err) => (Status::BadRequest, Json(Value::String(err.to_string()))),
    }
}

#[get("/<blog_id>")]
fn read(blog_id: u32, db_conn: &State<DbConnection>) -> HandlerResult {
    match db_conn.0.lock().expect("Poisoned lock").query_one(
        "SELECT * FROM blogs WHERE id = ?1 AND deleted_at IS NULL",
        [blog_id],
        |row| {
            let map: Map<String, Value> = [
                ("id", Value::from(row.get::<_, u32>(0)?)),
                ("title", Value::from(row.get::<_, String>(1)?)),
                ("content", Value::from(row.get::<_, String>(2)?)),
                ("category", Value::from(row.get::<_, String>(3)?)),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

            Ok(Value::Object(map))
        },
    ) {
        Ok(json_val) => Ok((Status::Ok, Json(json_val))),
        Err(_) => Err((
            Status::BadRequest,
            Json(Value::String(
                format!("There is no blog with id: {blog_id}").to_string(),
            )),
        )),
    }
}

#[delete("/<blog_id>")]
fn delete(blog_id: u32, db_conn: &State<DbConnection>) -> HandlerResult {
    match db_conn.0.lock().expect("Poisoned lock").execute(
        "UPDATE blogs SET deleted_at = datetime('now') WHERE id = ?1 AND deleted_at IS NULL",
        [blog_id],
    ) {
        Ok(_) => Ok((Status::Ok, Json(json!({"success": true})))),
        Err(err) => Err((
            Status::InternalServerError,
            Json(Value::String(err.to_string())),
        )),
    }
}

#[get("/?<term>")]
fn read_all_blogs_with_term(term: &str, db_conn: &State<DbConnection>) -> HandlerResult {
    let conn = db_conn.0.lock().expect("Poisoned lock");

    match conn.query_one(
        "SELECT title, content, category FROM blogs WHERE title LIKE ?1 AND deleted_at IS NULL",
        [format!("%{term}%")],
        |row| {
            let json_value = json!({
                "title": row.get_unwrap::<usize,String>(0),
                "content": row.get_unwrap::<usize,String>(1),
                "category": row.get_unwrap::<usize,String>(2),
            });

            Ok(json_value)
        },
    ) {
        Ok(value) => Ok((Status::Ok, Json(value))),
        Err(err) => Err((Status::BadRequest, Json(Value::String(err.to_string())))),
    }
}

#[get("/")]
fn read_all_blogs(
    db_conn: &State<DbConnection>,
) -> Result<(Status, Json<Value>), (Status, Json<Value>)> {
    let conn = db_conn.0.lock().expect("Poisoned lock");

    match conn.prepare("SELECT title, content, category FROM blogs WHERE deleted_at IS NULL") {
        Ok(mut stmt) => {
            let rows: Vec<Value> = stmt
                .query_and_then([], |row| row.get::<usize, String>(0))
                .unwrap()
                .map(|item| Value::String(item.unwrap_or_default()))
                .collect();

            Ok((Status::Ok, Json(Value::Array(rows))))
        }
        Err(err) => {
            let e = err.to_string();
            println!("{e:?}");

            Err((Status::BadRequest, Json(Value::String(e))))
        }
    }
}

#[put("/<blog_id>", format = "json", data = "<blog_json>")]
fn update(
    blog_json: validate::Validated<models::Blog>,
    blog_id: u32,
    db_conn: &State<DbConnection>,
) -> HandlerResult {
    let blog = blog_json.0.on_update();

    match db_conn.0.lock().expect("Poisoned lock").query_one(
        "UPDATE blogs SET title = ?1, content = ?2, category = ?3 WHERE id = ?4 AND deleted_at IS NULL RETURNING title, content, category",
        [
            &blog.title,
            &blog.content,
            &blog.category,
            &blog_id.to_string(),
        ],
        |row| {
            let json_value = json!({
                "title": row.get_unwrap::<usize,String>(0),
                "content": row.get_unwrap::<usize,String>(1),
                "category": row.get_unwrap::<usize,String>(2),
            });

            Ok(json_value)
        },
    ) {
        Ok(json_value) => Ok((Status::Ok, Json(json_value))),
        Err(err) => Err((Status::InternalServerError, Json(Value::String(err.to_string())))),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(BlogCount(AtomicU32::new(1)))
        .attach(db::stage())
        .mount(
            "/posts",
            routes![
                read_all_blogs,
                read_all_blogs_with_term,
                create,
                read,
                update,
                delete
            ],
        )
}

#[cfg(test)]
mod tests;
