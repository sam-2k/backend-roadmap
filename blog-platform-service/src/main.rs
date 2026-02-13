#[allow(non_snake_case)]
#[macro_use]
extern crate rocket;

use std::{fs::remove_file, sync::atomic::AtomicU32};

use rocket::{
    State,
    http::Status,
    serde::json::{Json, Value, to_value},
};
use serde_json::{Map, json};

use crate::db::DbConnection;

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

    if let Ok(_) = db_conn.0.lock().expect("Lock is poisoned").execute(
        "INSERT INTO blogs (title, content, category) VALUES (?1, ?2, ?3)",
        (&blog.title, &blog.content, &blog.category),
    ) {
        match to_value(blog) {
            Ok(value) => (Status::Ok, Json(value)),
            Err(_) => (Status::BadRequest, Json(Value::String("error".to_string()))),
        }
    } else {
        (
            Status::BadRequest,
            Json(Value::String("Cant insert data into table".to_string())),
        )
    }
}

#[get("/<blog_id>")]
fn read_blog(blog_id: u32, db_conn: &State<DbConnection>) -> HandlerResult {
    match db_conn.0.lock().expect("Poisoned lock").query_one(
        "SELECT * FROM blogs WHERE id = ?1",
        [blog_id],
        |row| {
            let mut map: Map<String, Value> = [
                ("id", Value::from(row.get::<_, u32>(0)?)),
                ("title", Value::from(row.get::<_, String>(1)?)),
                ("content", Value::from(row.get::<_, String>(2)?)),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

            if let Ok(cat) = row.get::<_, String>(3) {
                map.insert("category".to_string(), Value::from(cat));
            }

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
fn delete(blog_id: usize) -> Result<Status, (Status, Value)> {
    match remove_file(format!("uploads/blog-{blog_id}.json")) {
        Ok(()) => Ok(Status::NoContent),
        Err(_) => Err((Status::NotFound, Value::Null)),
    }
}

#[get("/?<term>")]
fn read_all_blogs_with_term(term: &str, db_conn: &State<DbConnection>) -> HandlerResult {
    let conn = db_conn.0.lock().expect("Poisoned lock");

    match conn.query_one(
        "SELECT title, content, category FROM blogs WHERE title LIKE ?1",
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

    match conn.prepare("SELECT title, content, category FROM blogs") {
        Ok(mut stmt) => {
            let rows: Vec<Value> = stmt
                .query_and_then([], |row| row.get::<usize, String>(0))
                .unwrap()
                .map(|item| Value::String(item.unwrap()))
                .collect();

            Ok((Status::Ok, Json(Value::Array(rows))))
        }
        Err(err) => Err((Status::BadRequest, Json(Value::String(err.to_string())))),
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
        "UPDATE blogs SET title = ?1, content = ?2, category = ?3 WHERE id = ?4 RETURNING title, content, category",
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
                create,
                read_blog,
                read_all_blogs,
                read_all_blogs_with_term,
                delete,
                update
            ],
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use serde_json::{Value, json};

    // Helper to build the Rocket instance for testing
    fn setup_client() -> Client {
        // Ensure this matches your actual rocket() constructor
        Client::tracked(crate::rocket()).expect("valid rocket instance")
    }

    #[test]
    fn test_create_and_read_lifecycle() {
        let client = setup_client();

        // 1. Define the blog payload
        // This must match your models::Blog fields for validation to pass
        let new_blog = json!({
            "title": "My First Post",
            "content": "Hello World!",
            "category": "Rust",
            "tags": ["testing"]
        });

        // 2. Test POST /posts (Create)
        let post_response = client
            .post("/posts")
            .header(ContentType::JSON)
            .body(new_blog.to_string())
            .dispatch();

        assert_eq!(post_response.status(), Status::Ok);

        let created_json: Value = post_response.into_json().expect("Response should be JSON");
        let new_id = created_json["id"].as_u64().expect("Should return an ID");

        // 3. Test GET /posts/<id> (Read)
        let get_url = format!("/posts/{}", new_id);
        let get_response = client.get(get_url).dispatch();

        assert_eq!(get_response.status(), Status::Ok);

        let fetched_json: Value = get_response.into_json().expect("Response should be JSON");
        assert_eq!(fetched_json["title"], "My First Post");
        assert_eq!(fetched_json["id"], new_id);
    }

    #[test]
    fn test_read_missing_blog() {
        let client = setup_client();

        // Testing the error arm of your match statement
        let response = client.get("/posts/999999").dispatch();

        assert_eq!(response.status(), Status::BadRequest);

        let error_msg: Value = response.into_json().unwrap();
        assert!(
            error_msg
                .as_str()
                .unwrap()
                .contains("There is no blog with id: 999999")
        );
    }

    #[test]
    fn test_read_all_blogs() {
        let client = setup_client();

        let new_blog = json!({
            "title": "My First Post",
            "content": "Hello World!",
            "category": "Rust",
            "tags": ["testing"]
        });

        // 2. Test POST /posts (Create)
        client
            .post("/posts")
            .header(ContentType::JSON)
            .body(new_blog.to_string())
            .dispatch();

        let new_blog = json!({
            "title": "My Second Post",
            "content": "Hello World v2!",
            "category": "Rust",
            "tags": ["testing"]
        });

        // 2. Test POST /posts (Create)
        client
            .post("/posts")
            .header(ContentType::JSON)
            .body(new_blog.to_string())
            .dispatch();

        // Testing the error arm of your match statement
        let response = client.get("/posts").dispatch();

        // assert_eq!(response.status(), Status::Ok);

        let fetched_json: Value = response.into_json().expect("Response should be JSON");
        println!("{fetched_json}");
    }

    #[test]
    fn test_put_blog() {
        let client = setup_client();

        // 1. Define the blog payload
        // This must match your models::Blog fields for validation to pass
        let new_blog = json!({
            "title": "My First Post",
            "content": "Hello World!",
            "category": "Rust",
            "tags": ["testing"]
        });

        // 2. Test POST /posts (Create)
        let post_response = client
            .post("/posts")
            .header(ContentType::JSON)
            .body(new_blog.to_string())
            .dispatch();

        assert_eq!(post_response.status(), Status::Ok);

        let created_json: Value = post_response.into_json().expect("Response should be JSON");
        let new_id = created_json["id"].as_u64().expect("Should return an ID");

        // 3. Test GET /posts/<id> (Read)
        let get_url = format!("/posts/{}", new_id);
        let get_response = client.get(get_url).dispatch();

        assert_eq!(get_response.status(), Status::Ok);

        let fetched_json: Value = get_response.into_json().expect("Response should be JSON");
        assert_eq!(fetched_json["title"], "My First Post");
        assert_eq!(fetched_json["id"], new_id);

        let new_blog = json!({
            "title": "(Not) My First Post",
            "content": "Hello World!",
            "category": "Rust",
            "tags": ["testing"]
        });

        // 2. Test POST /posts (Create)
        let put_response = client
            .put(format!("/posts/{}", new_id))
            .header(ContentType::JSON)
            .body(new_blog.to_string())
            .dispatch();

        let created_json: Value = put_response.into_json().expect("Response should be JSON");
        let new_title = created_json["title"].as_str().expect("Should return an ID");

        assert_eq!(new_title, "(Not) My First Post");
    }

    #[test]
    fn test_blog_with_term() {
        let client = setup_client();

        assert!(
            [
                client
                    .post("/posts")
                    .header(ContentType::JSON)
                    .body(
                        json!({
                            "title": "My First Post",
                            "content": "Hello World!",
                            "category": "Rust",
                            "tags": ["testing"]
                        })
                        .to_string(),
                    )
                    .dispatch(),
                client
                    .post("/posts")
                    .header(ContentType::JSON)
                    .body(
                        json!({
                            "title": "My Second Post",
                            "content": "Hello World!",
                            "category": "Rust",
                            "tags": ["testing"]
                        })
                        .to_string(),
                    )
                    .dispatch(),
            ]
            .iter()
            .all(|res| res.status() == Status::Ok)
        );

        let get_response = client
            .get(format!("/posts?term={}", "Second"))
            .header(ContentType::JSON)
            .dispatch();

        assert_eq!(get_response.status(), Status::Ok);
        if let Some(v) = get_response.into_json::<Value>().take() {
            assert_eq!(v["title"].as_str(), Some("My Second Post"));
        } else {
            panic!("Can't parse json");
        }
    }
}
