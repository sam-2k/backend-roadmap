use std::sync::{Arc, Mutex};

use rocket::{
    State,
    fairing::AdHoc,
    serde::json::{Json, Value, to_value},
};
use rusqlite::Connection;

pub struct DbConnection(pub Arc<Mutex<Connection>>);

#[post("/")]
fn index(db_conn: &State<DbConnection>) -> String {
    let conn = db_conn.0.lock().unwrap();

    match conn.execute(
        "INSERT INTO person (name, data) VALUES (?1, ?2)",
        ("Sam", "Some Data"),
    ) {
        Ok(_) => "Hello db".to_string(),
        Err(_) => "Failed".to_string(),
    }
}

#[get("/")]
fn read_all_users(db_conn: &State<DbConnection>) -> Json<Value> {
    let conn = db_conn.0.lock().unwrap();

    let mut stmt = conn.prepare("SELECT id, name, data FROM person").unwrap();
    // 1. Map the rows to a specific type (String in your case for 'name')
    let person_iter = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            Ok(name)
        })
        .unwrap();

    // 2. Collect the results into a Vec<String>
    // Using .collect::<Result<Vec<_>, _>>() will stop and return an error if any row fails
    let names: Vec<String> = person_iter
        .filter_map(|res| res.ok()) // Simplest way: skip errors
        .collect();

    return Json(to_value(names).unwrap());
}

pub fn stage() -> AdHoc {
    let conn: Option<Connection> = Connection::open_in_memory().ok();

    if let Some(conn) = &conn {
        conn.execute(
            "CREATE TABLE blogs (
                  id    INTEGER PRIMARY KEY,
                  title  TEXT NOT NULL,
                  content  TEXT NOT NULL,
                  category TEXT NOT NULL
              )",
            (), // empty list of parameters.
        )
        .expect("Maybe SQL syntax wrong");
    }

    AdHoc::on_ignite("database", |rocket| async {
        rocket
            .manage(DbConnection(Arc::new(Mutex::new(conn.unwrap()))))
            .mount("/db", routes![index, read_all_users])
    })
}
