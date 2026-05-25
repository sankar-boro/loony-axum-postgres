use crate::error::AppError;
use crate::AppState;
use axum::{
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;

// ---------------------------------------------------------------------------
// In-memory representation of the parsed ZIP structure
// ---------------------------------------------------------------------------

#[derive(Default)]
struct BookStructure {
    title: String,
    content: String,
    chapters: BTreeMap<u32, ChapterData>,
}

#[derive(Default)]
struct ChapterData {
    title: String,
    content: String,
    sections: BTreeMap<u32, SectionData>,
}

#[derive(Default)]
struct SectionData {
    title: String,
    content: String,
    subsections: BTreeMap<u32, SubsectionData>,
}

struct SubsectionData {
    title: String,
    content: String,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// POST /book/upload
///
/// Accepts a multipart form with a single `file` field containing a ZIP
/// archive. The archive must follow this folder structure:
///
///   <root>/
///     cover.md            ← book cover page  (identity 100)
///     01-chapter-one/     ← chapter          (identity 101)
///       _index.md         ← optional chapter intro
///       01-section-a/     ← section          (identity 102)
///         01-intro.md     ← sub-section      (identity 103)
///         02-advanced.md
///       02-section-b/
///         01-overview.md
///     02-chapter-two/
///       ...
///
/// Files and folders are ordered by a numeric prefix (`NN-`). Folders
/// without a prefix are sorted after numbered entries (order = 999).
/// The root folder name (if the ZIP has one) is stripped automatically.
pub async fn upload_book(
    axum::extract::Extension(crate::utils::UserId(user_id)): axum::extract::Extension<
        crate::utils::UserId,
    >,
    State(pool): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // --- 1. Read ZIP bytes from the first multipart field ---
    let mut zip_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await? {
        zip_bytes = Some(field.bytes().await?.to_vec());
        break;
    }
    let zip_bytes =
        zip_bytes.ok_or_else(|| AppError::BadRequest("No file field in multipart body".into()))?;

    // --- 2. Parse ZIP into the in-memory BookStructure ---
    let book = parse_zip(&zip_bytes)?;

    if book.title.is_empty() {
        return Err(AppError::BadRequest(
            "ZIP must contain a cover.md / index.md / README.md at the root level".into(),
        ));
    }

    // --- 3. Bulk-insert in a single transaction ---
    let mut conn = pool.pg_pool.conn.get().await?;
    let empty_images = "[]";

    let tx = conn.transaction().await?;

    // Insert the `books` summary row
    let row = tx
        .query_one(
            "INSERT INTO books(user_id, title, content, images) \
             VALUES($1, $2, $3, $4) RETURNING uid",
            &[&user_id, &book.title, &book.content, &empty_images],
        )
        .await?;
    let doc_id: i32 = row.get(0);

    // Insert the front-page node (identity = 100)
    let identity_100: i16 = 100;
    let row = tx
        .query_one(
            "INSERT INTO book(user_id, doc_id, title, content, identity, images) \
             VALUES($1, $2, $3, $4, $5, $6) RETURNING uid",
            &[
                &user_id,
                &doc_id,
                &book.title,
                &book.content,
                &identity_100,
                &empty_images,
            ],
        )
        .await?;
    let main_uid: i32 = row.get(0);

    let identity_101: i16 = 101;
    let identity_102: i16 = 102;
    let identity_103: i16 = 103;

    // Prepared statement reused for all child nodes
    let insert_node = tx
        .prepare(
            "INSERT INTO book(user_id, doc_id, page_id, parent_id, title, content, identity, images) \
             VALUES($1, $2, $3, $4, $5, $6, $7, $8) RETURNING uid",
        )
        .await?;

    let mut prev_chapter_uid = main_uid;

    for chapter in book.chapters.values() {
        // Insert chapter (101); parent_id walks the chapter linked-list
        let row = tx
            .query_one(
                &insert_node,
                &[
                    &user_id,
                    &doc_id,
                    &main_uid,         // page_id = front page
                    &prev_chapter_uid, // parent_id = previous chapter (or main for first)
                    &chapter.title,
                    &chapter.content,
                    &identity_101,
                    &empty_images,
                ],
            )
            .await?;
        let chapter_uid: i32 = row.get(0);

        let mut prev_section_uid = chapter_uid;

        for section in chapter.sections.values() {
            // Insert section (102); page_id = chapter
            let row = tx
                .query_one(
                    &insert_node,
                    &[
                        &user_id,
                        &doc_id,
                        &chapter_uid,      // page_id = chapter
                        &prev_section_uid, // parent_id = previous section (or chapter for first)
                        &section.title,
                        &section.content,
                        &identity_102,
                        &empty_images,
                    ],
                )
                .await?;
            let section_uid: i32 = row.get(0);

            let mut prev_sub_uid = section_uid;

            for sub in section.subsections.values() {
                // Insert sub-section (103); page_id = section
                let row = tx
                    .query_one(
                        &insert_node,
                        &[
                            &user_id,
                            &doc_id,
                            &section_uid, // page_id = section
                            &prev_sub_uid, // parent_id = previous sub (or section for first)
                            &sub.title,
                            &sub.content,
                            &identity_103,
                            &empty_images,
                        ],
                    )
                    .await?;
                prev_sub_uid = row.get(0);
            }

            prev_section_uid = section_uid;
        }

        prev_chapter_uid = chapter_uid;
    }

    tx.commit().await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "doc_id": doc_id,
            "title": book.title,
            "chapters": book.chapters.len(),
        })),
    ))
}

// ---------------------------------------------------------------------------
// ZIP parsing
// ---------------------------------------------------------------------------

fn parse_zip(bytes: &[u8]) -> Result<BookStructure, AppError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| AppError::BadRequest(format!("Invalid ZIP file: {e}")))?;

    // Collect (path_parts, file_content) for every .md file
    let mut files: Vec<(Vec<String>, String)> = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if entry.is_dir() {
            continue;
        }

        let raw_name = entry.name().to_string();
        if !raw_name.ends_with(".md") {
            continue;
        }

        let mut content = String::new();
        entry
            .read_to_string(&mut content)
            .map_err(|e| AppError::BadRequest(format!("Cannot read {raw_name}: {e}")))?;

        let parts: Vec<String> = raw_name
            .split('/')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        files.push((parts, content));
    }

    if files.is_empty() {
        return Err(AppError::BadRequest(
            "ZIP contains no .md files".into(),
        ));
    }

    // Strip common root folder (many ZIP tools wrap everything in one folder)
    let strip = has_single_root(&files);
    let files: Vec<(Vec<String>, String)> = files
        .into_iter()
        .map(|(parts, content)| {
            let start = if strip { 1 } else { 0 };
            (parts[start..].to_vec(), content)
        })
        .collect();

    let mut book = BookStructure::default();

    for (parts, content) in &files {
        match parts.as_slice() {
            // ── Root-level .md ──────────────────────────────────────────────
            [filename] => {
                let lower = filename.to_lowercase();
                if matches!(lower.as_str(), "cover.md" | "index.md" | "readme.md") {
                    let (title, body) = split_title_body(content, &title_from_name(filename));
                    book.title = title;
                    book.content = body;
                }
                // introduction.md and other root files are silently ignored
            }

            // ── Chapter-level file (e.g. 01-intro/_index.md) ───────────────
            [chapter_dir, filename] => {
                let (ch_order, ch_title) = order_and_title(chapter_dir);
                let chapter = book.chapters.entry(ch_order).or_insert_with(|| ChapterData {
                    title: ch_title.clone(),
                    ..Default::default()
                });
                if is_index(filename) {
                    let (_, body) = split_title_body(content, &ch_title);
                    chapter.content = body;
                }
                // Non-index files at chapter level are ignored
            }

            // ── Section file (e.g. 01-intro/01-basics/01-hello.md) ─────────
            [chapter_dir, section_dir, filename] => {
                let (ch_order, ch_title) = order_and_title(chapter_dir);
                let (sec_order, sec_title) = order_and_title(section_dir);

                let chapter = book.chapters.entry(ch_order).or_insert_with(|| ChapterData {
                    title: ch_title,
                    ..Default::default()
                });

                let section = chapter.sections.entry(sec_order).or_insert_with(|| SectionData {
                    title: sec_title.clone(),
                    ..Default::default()
                });

                if is_index(filename) {
                    let (_, body) = split_title_body(content, &sec_title);
                    section.content = body;
                } else {
                    let (sub_order, sub_title) = order_and_title(filename);
                    let (_, body) = split_title_body(content, &sub_title);
                    section.subsections.entry(sub_order).or_insert(SubsectionData {
                        title: sub_title,
                        content: body,
                    });
                }
            }

            // Deeper nesting is ignored
            _ => {}
        }
    }

    // Fallback title if no cover file was present
    if book.title.is_empty() {
        book.title = book
            .chapters
            .values()
            .next()
            .map(|c| c.title.clone())
            .unwrap_or_else(|| "Untitled Book".into());
        book.content = format!("<basic> {}", book.title);
    }

    Ok(book)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true if every file path shares the same top-level component,
/// meaning the ZIP was created with a root folder wrapper.
fn has_single_root(files: &[(Vec<String>, String)]) -> bool {
    if files.is_empty() {
        return false;
    }
    let first = match files[0].0.first() {
        Some(f) => f,
        None => return false,
    };
    files.iter().all(|(parts, _)| parts.first() == Some(first) && parts.len() > 1)
}

/// Parse a numeric prefix (`NN-` or `NN_`) from a file/folder name and
/// return `(order, human_title)`. Names without a prefix get order 999.
fn order_and_title(raw: &str) -> (u32, String) {
    // Strip .md extension if present
    let name = raw.strip_suffix(".md").unwrap_or(raw);

    if let Some(sep) = name.find(|c: char| c == '-' || c == '_') {
        let prefix = &name[..sep];
        if let Ok(order) = prefix.parse::<u32>() {
            let rest = &name[sep + 1..];
            return (order, to_title_case(rest));
        }
    }
    (999, to_title_case(name))
}

/// Convert a slug like `getting-started` to `Getting Started`.
fn to_title_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_')
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Derive a title from a raw filename (no extension, title-cased).
fn title_from_name(filename: &str) -> String {
    let name = filename.strip_suffix(".md").unwrap_or(filename);
    to_title_case(name)
}

/// Returns true for `_index.md`, `index.md`, `README.md` (case-insensitive).
fn is_index(filename: &str) -> bool {
    matches!(
        filename.to_lowercase().as_str(),
        "_index.md" | "index.md" | "readme.md"
    )
}

/// If the content starts with a Markdown H1 (`# Title`), extract it as the
/// node title and return the remaining body. Otherwise keep `default_title`.
/// The body is prefixed with `<basic> ` to signal the basic markdown renderer.
fn split_title_body(content: &str, default_title: &str) -> (String, String) {
    let trimmed = content.trim();
    if let Some(rest) = trimmed.strip_prefix("# ") {
        let nl = rest.find('\n').unwrap_or(rest.len());
        let title = rest[..nl].trim().to_string();
        let body = rest[nl..].trim().to_string();
        return (title, format!("<basic> {body}"));
    }
    (default_title.to_string(), format!("<basic> {trimmed}"))
}
