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

    assert_eq!(response.status(), Status::Ok);

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

#[test]
fn test_delete_blog() {
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
            client
                .post("/posts")
                .header(ContentType::JSON)
                .body(
                    json!({
                        "title": "My 3rd Post",
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

    let delete_response = client
        .delete("/posts/1")
        .header(ContentType::Text)
        .dispatch();

    assert_eq!(delete_response.status(), Status::Ok);
    if let Some(v) = delete_response.into_json::<Value>().take() {
        assert_eq!(v["success"].as_bool(), Some(true));
    } else {
        panic!("Can't parse json");
    }

    let get_all_blogs_response = client
        .get(format!("/posts/"))
        .header(ContentType::JSON)
        .dispatch();

    assert_eq!(get_all_blogs_response.status(), Status::Ok);
    if let Some(v) = get_all_blogs_response.into_json::<Value>().take() {
        println!("{v:?}");

        assert_eq!(v.as_array().unwrap_or(&Vec::new()).iter().count(), 2);
    } else {
        panic!("Can't parse json");
    }
}
