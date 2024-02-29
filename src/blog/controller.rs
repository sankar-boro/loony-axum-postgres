use crate::query::{BOOK_DATA};
use deadpool_postgres::Pool;
use actix_session::Session;
use actix_web::{HttpResponse, web};
use serde_json::json;
use crate::error::Error;
use super::model::{GetBlog, GetBlogs};

pub static BLOGS: &str = "SELECT uid, authorid, title, body, metadata, createdat FROM blog";
pub async fn get_all_blogs(
    app: web::Data<Pool>,
    _: Session
) 
-> Result<HttpResponse, Error> 
{
    let conn = app.get().await?;
    let blogs = conn.query(
        BLOGS, 
        &[]
    ).await?;

    let mut allblogs = Vec::new();
    for i in 0..blogs.len() {
        allblogs.push(GetBlogs {
            uid: blogs[i].get(0),
            authorid: blogs[i].get(1),
            title: blogs[i].get(2),
            body: blogs[i].get(3),
            metadata: blogs[i].get(4),
            createdat: blogs[i].get(5)
        });
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "data": allblogs
    })))
}

pub async fn get_all_blog_nodes(
    app: web::Data<Pool>,
    path: web::Path<String>,
    _: Session
) 
-> Result<HttpResponse, Error> 
{
	let docid: i32 = path.parse()?;
    let conn = app.get().await?;
    let blogs = conn.query(
        BOOK_DATA, 
        &[&docid]
    ).await?;

    let mut allblogs = Vec::new();
    for i in 0..blogs.len() {
        allblogs.push(GetBlog {
            uid: blogs[i].get(0),
            authorid: blogs[i].get(1),
            docid: blogs[i].get(2),
            parentid: blogs[i].get(3),
            title: blogs[i].get(4),
            body: blogs[i].get(5),
            identity: blogs[i].get(6),
            metadata: blogs[i].get(7),
            createdat: blogs[i].get(8)
        });

    }

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "data": allblogs
    })))
}