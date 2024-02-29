use deadpool_postgres::Pool;
// use actix_session::Session;
use actix_web::{HttpResponse, web};
use serde_json::json;
use crate::error::Error;
use serde::Serialize;

#[derive(Serialize)]
pub struct GetNodes {
    pub uid: i32,
    pub authorid: i32,
    pub parentid: i32,
    pub title: String,
    pub body: String,
    pub identity: i16,
    pub metadata: String,
    pub createdat: std::time::SystemTime,
}

pub static NODES: &str = "SELECT uid, authorid, parentid, title, body, identity, metadata, createdat FROM booknode where docid=$1 AND pageid=$2";
pub async fn nodes(
    app: web::Data<Pool>,
    path: web::Path<(String, String)>
) 
-> Result<HttpResponse, Error> 
{
  let docid: i32 = path.0.parse()?;
  let pageid: i32 = path.1.parse()?;

    let conn = app.get().await?;
    let books = conn.query(
      NODES, 
        &[&docid, &pageid]
    ).await?;

    let mut allbooks = Vec::new();
    for i in 0..books.len() {
        allbooks.push(GetNodes {
            uid: books[i].get(0),
            authorid: books[i].get(1),
            parentid: books[i].get(2),
            title: books[i].get(3),
            body: books[i].get(4),
            identity: books[i].get(5),
            metadata: books[i].get(6),
            createdat: books[i].get(7)
        });
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "data": allbooks
    })))
}

