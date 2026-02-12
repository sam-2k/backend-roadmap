#[allow(non_snake_case)]
#[macro_use]
extern crate rocket;

use std::{
    fs::{create_dir_all, exists, read_dir, read_to_string, remove_file, write},
    sync::atomic::AtomicUsize,
};

use rocket::{
    State,
    fairing::AdHoc,
    http::Status,
    serde::{
        Deserialize, Serialize,
        json::{Json, Value, from_str, to_string},
    },
};

const UPLOAD_DIR: &'static str = "uploads";

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct BlogRequest {
    title: String,
    content: String,
    category: String,
    tags: Vec<String>,
}

#[allow(nonstandard_style)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Blog {
    id: usize,
    title: String,
    content: String,
    category: String,
    tags: Vec<String>,
    createdAt: String,
    updatedAt: String,
}

impl BlogRequest {
    fn validate(&self) -> bool {
        [&self.title, &self.content, &self.category]
            .iter()
            .any(|s| !s.is_empty())
            && !self.tags.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct ErrorMessage {
    message: String,
}

type ErrorResponse = (Status, Json<ErrorMessage>);
struct BlogCount(AtomicUsize);

mod time;

#[post("/", format = "json", data = "<blog>")]
fn create(
    blog: Json<BlogRequest>,
    blog_count: &State<BlogCount>,
) -> Result<(Status, Json<Blog>), ErrorResponse> {
    let blog: BlogRequest = blog.into_inner();

    let blog_id = blog_count.0.load(std::sync::atomic::Ordering::Relaxed);
    blog_count
        .0
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Get the current time in UTC
    let created_at = time::get_now();
    let updated_at = created_at.clone();

    if !blog.validate() {
        return Err((
            Status::BadRequest,
            Json(ErrorMessage {
                message: "Please make sure all fields is filled".to_string(),
            }),
        ));
    }

    let json_blog = Json(Blog {
        title: blog.title,
        content: blog.content,
        category: blog.category,
        tags: blog.tags,
        updatedAt: updated_at,
        createdAt: created_at,
        id: blog_id,
    });

    match write(
        format!("{UPLOAD_DIR}/blog-{blog_id}.json"),
        to_string(&json_blog.0).unwrap(),
    ) {
        Ok(()) => return Ok((Status::Created, json_blog)),
        Err(err) => {
            return Err((
                Status::BadRequest,
                Json(ErrorMessage {
                    message: err.to_string(),
                }),
            ));
        }
    }
}

#[delete("/<blog_id>")]
fn delete(blog_id: usize) -> Result<Status, ErrorResponse> {
    match remove_file(format!("uploads/blog-{blog_id}.json")) {
        Ok(()) => Ok(Status::NoContent),
        Err(_) => Err((
            Status::NotFound,
            Json(ErrorMessage {
                message: "The blog was not found !".to_string(),
            }),
        )),
    }
}

#[get("/<blog_id>")]
fn read_blog(blog_id: usize) -> Result<(Status, Value), ErrorResponse> {
    match read_to_string(format!("uploads/blog-{blog_id}.json")) {
        Ok(json_string) => {
            let json_blog = from_str::<Value>(&json_string).unwrap();

            Ok((Status::Ok, json_blog))
        }
        Err(_) => Err((
            Status::NotFound,
            Json(ErrorMessage {
                message: "The blog was not found !".to_string(),
            }),
        )),
    }
}

#[get("/?<term>")]
fn read_all_blogs_with_term(term: &str) -> Result<(Status, Json<Vec<Blog>>), ErrorResponse> {
    if let Ok(entry) = read_dir(format!("uploads")) {
        let mut vec: Vec<Blog> = vec![];

        println!("term {term}");

        for entry in entry {
            if let Ok(entry) = entry {
                let blog: Blog = from_str(&read_to_string(entry.path()).unwrap()).unwrap();
                if blog.title.contains(term)
                    || blog.category.contains(term)
                    || blog.tags.contains(&String::from(term))
                    || blog.content.contains(term)
                {
                    vec.push(blog);
                }
            }
        }

        Ok((Status::Ok, Json(vec)))
    } else {
        Err((
            Status::NotFound,
            Json(ErrorMessage {
                message: "The blog was not found !".to_string(),
            }),
        ))
    }
}

#[get("/")]
fn read_all_blogs() -> Result<(Status, Json<Vec<Blog>>), ErrorResponse> {
    if let Ok(entry) = read_dir(format!("uploads")) {
        let mut vec: Vec<Blog> = vec![];

        for entry in entry {
            if let Ok(entry) = entry {
                let blog: Blog = from_str(&read_to_string(entry.path()).unwrap()).unwrap();
                vec.push(blog);
            }
        }

        Ok((Status::Ok, Json(vec)))
    } else {
        Err((
            Status::NotFound,
            Json(ErrorMessage {
                message: "The blog was not found !".to_string(),
            }),
        ))
    }
}

#[put("/<blog_id>", format = "json", data = "<blog>")]
fn update(blog: Json<BlogRequest>, blog_id: usize) -> Result<(Status, Json<Blog>), ErrorResponse> {
    let path_buf = format!("{UPLOAD_DIR}/blog-{blog_id}.json");
    let original_blog: Blog = from_str(read_to_string(&path_buf).unwrap().as_str()).unwrap();
    let updated_blog: Blog = Blog {
        id: original_blog.id,
        title: blog.title.clone(),
        content: blog.content.clone(),
        tags: blog.tags.clone(),
        category: blog.category.clone(),
        createdAt: original_blog.createdAt,
        updatedAt: time::get_now(),
    };

    if !blog.validate() {
        if let Ok(_) =
            exists(&path_buf).and_then(|_| write(&path_buf, to_string(&updated_blog).unwrap()))
        {
            Ok((Status::Ok, Json(updated_blog)))
        } else {
            Err((
                Status::NotFound,
                Json(ErrorMessage {
                    message: "The blog was not found".to_string(),
                }),
            ))
        }
    } else {
        Err((
            Status::BadRequest,
            Json(ErrorMessage {
                message: "The content was invalid".to_string(),
            }),
        ))
    }
}

#[launch]
fn rocket() -> _ {
    let init_blog_count = read_dir(format!("{UPLOAD_DIR}")).map_or(1, |v| v.count());

    rocket::build()
        .manage(BlogCount(AtomicUsize::new(init_blog_count)))
        .attach(AdHoc::on_ignite("upload directory", |rocket| async {
            create_dir_all(format!("{UPLOAD_DIR}"))
                .expect("Something wrong when create `uploads` directory");
            rocket
        }))
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
