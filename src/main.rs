#[macro_use]
extern crate rocket;

use std::{io::{Error, ErrorKind}, sync::Arc};

use rocket::{serde::json::Json, State};
use surrealdb::{sql::Object, kvs::Datastore, dbs::Session, iam::{Level, Role}};

use crate::db::{AffectedRows, DB};

use cors::*;

mod error;
mod prelude;
mod utils;
mod db;
mod cors;

#[post("/task/<title>")]
async fn add_task(title: String, db: &State<DB>) -> Result<Json<Object>, Error> {
    let task: Object = db
        .add_task(title)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "Unable to create task."))?;

    Ok(Json(task))
}

#[get("/task/<id>")]
async fn get_task(id: String, db: &State<DB>) -> Result<Json<Object>, Error> {
    let task: Object = db
        .get_task(id)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "Unable to fetch task."))?;

    Ok(Json(task))
}

#[get("/tasks")]
async fn get_all_tasks(db: &State<DB>) -> Result<Json<Vec<Object>>, Error> {
    let task: Vec<Object> = db
        .get_all_tasks()
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "Unable to fetch all tasks."))?;

    Ok(Json(task))
}

#[patch("/task/<id>")]
async fn toggle_task(id: String, db: &State<DB>) -> Result<Json<AffectedRows>, Error> {
    let affected_rows: AffectedRows = db
        .toggle_task(id)
        .await
        .map_err(|e: error::Error| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(Json(affected_rows))
}

#[delete("/task/<id>")]
async fn delete_task(id: String, db: &State<DB>) -> Result<Json<AffectedRows>, Error> {
    let affected_rows: AffectedRows = db
        .delete_task(id)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "Unable to delete task."))?;

    Ok(Json(affected_rows))
}

#[launch]
async fn rocket() -> _ {
    let ds: Arc<Datastore> = Arc::new(Datastore::new("memory").await.unwrap());
    // let sesh = Session::for_db("my_ns", "my_db");
    let level: Level = Level::Database("my_ns".to_string(), "my_db".to_string());
    let role: Role = Role::Owner;
    let sesh: Session = Session::for_level(level, role);

    let db: DB = DB { ds, sesh };

    rocket::build()
        .mount(
            "/",
            routes![add_task, get_task, get_all_tasks, toggle_task, delete_task],
        )
        .attach(CORS)
        .manage(db)
}