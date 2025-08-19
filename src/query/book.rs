#[macro_export]
macro_rules! fetch_books_by_user_id {
    ($conn:expr, $user_id:expr) => {{
        use tokio_postgres::Row;

        let query = "SELECT uid, user_id, title, images, created_at FROM books WHERE deleted_at IS NULL AND user_id = $1";
        let rows: Vec<Row> = $conn.query(query, &[&$user_id]).await?;

        let books = rows
            .iter()
            .map(|row| Book {
                uid: row.get(0),
                user_id: row.get(1),
                title: row.get(2),
                images: row.get(3),
                created_at: row.get(4),
            })
            .collect::<Vec<Book>>();

        Ok::<_, AppError>(books)
    }};
}

#[macro_export]
macro_rules! fetch_books_by_doc_ids {
    ($conn:expr, $doc_ids:expr) => {{
        use tokio_postgres::Row;
        
        let books_query = "SELECT uid, user_id, title, images, created_at FROM books where uid=ANY($1) AND deleted_at is NULL";
        let rows: Vec<Row> = $conn.query(books_query, &[&$doc_ids]).await?;

        let books = rows
            .iter()
            .map(|row| Book {
                uid: row.get(0),
                user_id: row.get(1),
                title: row.get(2),
                images: row.get(3),
                created_at: row.get(4),
            })
            .collect::<Vec<Book>>();

        Ok::<_, AppError>(books)
    }};
}

#[macro_export]
macro_rules! fetch_book_pages {
    ($conn:expr, $doc_id:expr) => {{
        use tokio_postgres::Row;
        let book_row: Row = $conn
        .query_one(
            "SELECT uid, user_id, title, content, images, created_at FROM book where doc_id=$1 and identity=100 and deleted_at is null LIMIT 1",
            &[&$doc_id],
        )
        .await?;

        let main_node = BookParentNode {
            uid: book_row.get(0),
            doc_id: $doc_id,
            user_id: book_row.get(1),
            title: book_row.get(2),
            content: book_row.get(3),
            images: book_row.get(4),
            created_at: book_row.get(5)
        };
        let rows = $conn
            .query(
                "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where doc_id=$1 AND identity IN(101, 102) and deleted_at is null",
                &[&$doc_id],
            )
            .await?;

        let books = rows
        .iter()
        .map(|row| NavNodes {
            uid:  row.get(0),
            parent_id: row.get(1),
            title: row.get(2),
            content: row.get(3),
            images: row.get(4),
            identity: row.get(5),
            page_id: row.get(6),
        })
        .collect::<Vec<NavNodes>>();

        Ok::<_, AppError>((books, main_node))
    }};
}

#[macro_export]
macro_rules! fetch_home_books {
    ($conn:expr) => {{
        use tokio_postgres::Row;
        
        let books_query = "SELECT uid, user_id, title, images, created_at FROM books where deleted_at is NULL LIMIT 5";
        let rows: Vec<Row> = $conn.query(books_query, &[]).await?;

        let books = rows
            .iter()
            .map(|row| Book {
                uid: row.get(0),
                user_id: row.get(1),
                title: row.get(2),
                images: row.get(3),
                created_at: row.get(4),
            })
            .collect::<Vec<Book>>();

        Ok::<_, AppError>(books)
    }};
}