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
    pub parentid: Option<i32>,
    pub title: String,
    pub body: String,
    pub identity: i16,
    pub metadata: String,
    pub createdat: std::time::SystemTime,
}

pub static NODES: &str = "SELECT uid, authorid, parentid, title, body, identity, metadata, createdat FROM blognode where docid=$1";
pub async fn nodes(
    app: web::Data<Pool>,
    path: web::Path<String>
) 
-> Result<HttpResponse, Error> 
{
  let docid: i32 = path.parse()?;

    let conn = app.get().await?;
    let blogs = conn.query(
      NODES, 
        &[&docid]
    ).await?;

    let mut allblogs = Vec::new();
    for i in 0..blogs.len() {
        allblogs.push(GetNodes {
            uid: blogs[i].get(0),
            authorid: blogs[i].get(1),
            parentid: blogs[i].get(2),
            title: blogs[i].get(3),
            body: blogs[i].get(4),
            identity: blogs[i].get(5),
            metadata: blogs[i].get(6),
            createdat: blogs[i].get(7)
        });
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "data": allblogs
    })))
}

