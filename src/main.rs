#![feature(proc_macro_hygiene, decl_macro)] // enables procedural macros to retrieve end-points

#[macro_use] extern crate rocket;

use std::os::macos::raw::stat;

use serde::{Deserialize, Serialize};
use rocket_contrib::json::Json;
use rusqlite::Connection;

#[derive(Serialize)]
struct ToDoList {
    items: Vec<ToDoItem>,
}

#[derive(Serialize)]
struct ToDoItem {
    id: i64,
    item: String,
}

struct StatusMessage {
    message: String,
}

#[get("/")] // decorator that gets the specific route
fn index() -> String {
    format!("Hello world!")
}

#[get("/todo")]
fn fetch_all_todo_items() -> Result<Json<ToDoList>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(String::from("Failed to connect to database"))
        },
    };

    let mut statement = match db_connection
    .prepare("select id, item from todo_list;") {
        Ok(statement) => statement,
        Err(_) => return Err("Failed to prepare query".into()),
    };

    let results = statement.query_map(
        [],|row| {
            Ok(ToDoItem {
                id: row.get(0)?,
                item: row.get(1)?,
            })
        });

    match results {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<ToDoItem>> = rows.collect();

            match collection {
                Ok(items) => Ok(Json(ToDoList { items })),
                Err(_) => Err("Could not collect items".into()),
            }
        },
        Err(_) => Err("Failed to fetch todo items".into()),
    }
}

#[post("/todo", format = "json", data = "<item>")]
fn add_todo_item(item: Json<String>) -> Result<Json<StatusMessage>, String> {
    let db_connection = match Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_) => {
            return Err(String::from("Failed to connect to database"));
        }
    };

    let mut statement =
        match db_connection.prepare("insert into todo_list (id, item) values (null, $1);") {
            Ok(statement) => statement,
            Err(_) => return Err("Failed to prepare query".into()),
        };
    let results = statement.execute(&[&item.0]);

    match results {
        Ok(rows_affected) => Ok(Json(StatusMessage {
            message: format!("{} rows inserted!", rows_affected),
        })),
        Err(_) => Err("Failed to insert todo item".into()),
    }
}

fn main() {
    {
        let db_connection = Connection::open("data.sqlite").unwrap();

        db_connection
            .execute(
                "create table if not exists todo_list (
                    id integer primary key,
                    item varchar(64) not null
                );",
                [],
            )
            .unwrap();
    }

    rocket::ignite().mount("/", routes![index, fetch_all_todo_items])
    .launch();
}