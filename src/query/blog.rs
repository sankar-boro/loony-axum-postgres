
#[macro_export]
macro_rules! fetch_home_blogs {
    ($conn:expr) => {{
        use tokio_postgres::Row;
        
        let blogs_query = "SELECT uid, user_id, title, images, created_at FROM blogs where deleted_at is NULL LIMIT 5";
        let rows: Vec<Row> = $conn.query(blogs_query, &[]).await?;

        let blogs = rows
            .iter()
            .map(|row| Blog {
                uid: row.get(0),
                user_id: row.get(1),
                title: row.get(2),
                images: row.get(3),
                created_at: row.get(4),
            })
            .collect::<Vec<Blog>>();

        Ok::<_, AppError>(blogs)
    }};
}

#[macro_export]
macro_rules! fetch_blogs_by_user_id {
    ($conn:expr, $user_id:expr) => {{
        use tokio_postgres::Row;

        let query = "SELECT uid, user_id, title, images, created_at FROM blogs WHERE deleted_at IS NULL AND user_id = $1";
        let rows: Vec<Row> = $conn.query(query, &[&$user_id]).await?;

        let blogs = rows
            .iter()
            .map(|row| Blog {
                uid: row.get(0),
                user_id: row.get(1),
                title: row.get(2),
                images: row.get(3),
                created_at: row.get(4),
            })
            .collect::<Vec<Blog>>();

        Ok::<_, AppError>(blogs)
    }};
}