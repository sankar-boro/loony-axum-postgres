# Loony Book — API Reference

**Base URL:** `http://localhost:8003`  
**Auth service:** `https://localhost:8000` (issues session cookies)

---

## Authentication

All write endpoints require a valid `access_token` cookie issued by the auth service. The cookie is set automatically on login and read by the `require_auth` middleware.

| Step | Endpoint | Notes |
|------|----------|-------|
| Sign up | `POST https://localhost:8000/signup` | Creates a new user account |
| Log in | `POST https://localhost:8000/login` | Sets `access_token` + `refresh_token` cookies |
| Log out | `POST https://localhost:8000/logout` | Clears session cookies |

Requests to protected endpoints without a valid `access_token` return `401 Unauthorized`.

---

## Blog

### Public — Read

#### `GET /blog/get/nodes`

Fetch the main node and all child nodes for a blog.

**Query params**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `doc_id` | integer | yes | The blog's document ID |

**Response `200`**
```json
{
  "main_node": {
    "uid": 1,
    "user_id": 7,
    "title": "Getting Started with Rust",
    "content": "<basic> Rust is a systems language...",
    "images": null,
    "created_at": "2024-01-15T10:00:00Z"
  },
  "child_nodes": [
    {
      "uid": 2,
      "doc_id": 1,
      "parent_id": 1,
      "title": "Installation",
      "content": "<basic> Install via rustup...",
      "images": null,
      "created_at": "2024-01-15T10:05:00Z"
    }
  ]
}
```

---

#### `GET /blog/get/:page_no/by_page`

Paginated list of all blogs (2 per page).

**Path params:** `page_no` — page number starting from 1.

**Response `200`** — array of blog summaries:
```json
[
  {
    "uid": 1,
    "title": "Getting Started with Rust",
    "content": "<basic> ...",
    "images": "[]",
    "created_at": "2024-01-15T10:00:00Z",
    "doc_type": 1
  }
]
```

---

#### `GET /blog/get/:uid/user_blogs`

All blogs written by a specific user.

**Path params:** `uid` — user ID.

---

#### `GET /blog/get/:user_id/get_users_blog`

All blogs written by a specific user (alternate path).

**Path params:** `user_id` — user ID.

---

#### `GET /blog/get/home_blogs`

Returns up to 5 recent blogs for the home feed.

---

### Protected — Write

> All write endpoints require the `access_token` cookie.

#### `POST /blog/create`

Create a new blog.

**Request body**
```json
{
  "title": "Getting Started with Rust",
  "content": "<basic> Rust is a systems programming language...",
  "images": [{ "name": "uuid-filename.jpg" }],
  "tags": ["rust", "programming"]
}
```

**Response `200`**
```json
{
  "doc_id": 42,
  "title": "Getting Started with Rust",
  "content": "<basic> ...",
  "images": "[]",
  "user_id": 7
}
```

---

#### `POST /blog/edit/main`

Edit the main (root) node of a blog.

**Request body**
```json
{
  "uid": 1,
  "doc_id": 42,
  "title": "Updated Title",
  "content": "<basic> Updated content...",
  "images": []
}
```

**Response `200`** — echoes the request body.

---

#### `POST /blog/edit/node`

Edit a child node.

**Request body**
```json
{
  "uid": 5,
  "doc_id": 42,
  "title": "Updated Section Title",
  "content": "<basic> Updated content...",
  "images": []
}
```

**Response `200`** — echoes the request body.

---

#### `POST /blog/append/node`

Append a new node after a given parent node. Automatically maintains the linked-list order.

**Request body**
```json
{
  "doc_id": 42,
  "parent_id": 3,
  "title": "New Section",
  "content": "<basic> Section content...",
  "images": [],
  "tags": ["optional"]
}
```

**Response `200`**
```json
{
  "new_node": {
    "uid": 6,
    "parent_id": 3,
    "title": "New Section",
    "content": "<basic> ...",
    "images": "[]",
    "tags": null
  },
  "update_node": { "uid": 4, "parent_id": 6 }
}
```

`update_node` is the node that previously pointed to `parent_id`; its `parent_id` was updated to maintain list order. `null` if no re-linking was needed.

---

#### `POST /blog/delete`

Soft-delete an entire blog (sets `deleted_at`).

**Request body**
```json
{ "doc_id": 42 }
```

**Response `200`**
```json
{ "data": "blog deleted" }
```

---

#### `POST /blog/delete/node`

Soft-delete a single blog node and optionally re-link the chain.

**Request body**
```json
{
  "delete_node": { "uid": 5 },
  "update_node": { "uid": 6, "parent_id": 3 }
}
```

`update_node` is optional. When provided, node `uid=6` will have its `parent_id` set to `3` (the deleted node's parent), maintaining list continuity.

**Response `200`** — echoes the request body.

---

## Book

### Public — Read

#### `GET /book/get/nav`

Fetch the front-page node and the full chapter + section navigation tree.

**Query params**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `doc_id` | integer | yes | The book's document ID |

**Response `200`**
```json
{
  "main_node": {
    "uid": 10,
    "user_id": 3,
    "doc_id": 5,
    "title": "The Complete Guide to PostgreSQL",
    "content": "<basic> ...",
    "images": null,
    "created_at": "2024-01-15T10:00:00Z"
  },
  "child_nodes": [
    {
      "uid": 11,
      "parent_id": 10,
      "title": "Introduction",
      "content": "<basic> ...",
      "identity": 101,
      "page_id": 10,
      "images": null
    },
    {
      "uid": 12,
      "parent_id": 11,
      "title": "Overview",
      "content": "<basic> ...",
      "identity": 102,
      "page_id": 11,
      "images": null
    }
  ]
}
```

`child_nodes` contains nodes with `identity` 101 (chapters) and 102 (sections). The frontend reconstructs the tree from `parent_id` links.

---

#### `GET /book/get/chapter`

Fetch a chapter node and its direct sub-sections (identity 103).

**Query params**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `doc_id` | integer | yes | Book ID |
| `page_id` | integer | yes | Chapter node UID |

**Response `200`**
```json
{
  "nodes": [
    { "uid": 11, "parent_id": 10, "title": "Introduction", "content": "<basic> ...", "identity": 101, "page_id": 10, "images": null },
    { "uid": 20, "parent_id": 11, "title": "Sub-section", "content": "<basic> ...", "identity": 103, "page_id": 11, "images": null }
  ]
}
```

---

#### `GET /book/get/section`

Fetch a section node and all its sub-sections (identity 103).

**Query params**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `doc_id` | integer | yes | Book ID |
| `page_id` | integer | yes | Section node UID |

**Response `200`** — same shape as `/book/get/chapter`.

---

#### `GET /book/get/:user_id/get_users_book`

All books written by a specific user.

**Path params:** `user_id` — user ID.

---

#### `GET /book/get/:page_no/by_page`

Paginated list of all books (2 per page).

**Path params:** `page_no` — page number starting from 1.

---

#### `GET /book/get/:uid/user_books`

All books by a specific user (alternate path).

---

#### `GET /book/get/home_books`

Returns up to 5 recent books for the home feed.

---

### Protected — Write

> All write endpoints require the `access_token` cookie.

#### `POST /book/create`

Create a new book with a front-page node (identity 100).

**Request body**
```json
{
  "title": "The Complete Guide to PostgreSQL",
  "content": "<basic> A comprehensive guide...",
  "images": [],
  "tags": ["postgresql", "database"]
}
```

**Response `200`**
```json
{
  "doc_id": 5,
  "user_id": 3,
  "title": "The Complete Guide to PostgreSQL",
  "content": "<basic> ...",
  "identity": 100,
  "images": "[]"
}
```

---

#### `POST /book/append/node`

Append a chapter (101), section (102), or sub-section (103) to a book. Automatically maintains linked-list order within each level.

**Node identity hierarchy**

| Identity | Type | Parent identity |
|----------|------|----------------|
| 100 | Front page | — (root) |
| 101 | Chapter | 100 |
| 102 | Section | 101 |
| 103 | Sub-section | 102 |

> Directly appending a sub-section (103) under a chapter (101) is not allowed and returns `500`.

**Request body**
```json
{
  "doc_id": 5,
  "parent_id": 10,
  "page_id": 10,
  "title": "Introduction to PostgreSQL",
  "content": "<basic> Overview content...",
  "images": [],
  "identity": 101,
  "parent_identity": 100,
  "tags": null
}
```

| Field | Description |
|-------|-------------|
| `parent_id` | UID of the node this new node follows in the linked list |
| `page_id` | UID of the containing "page" — for chapters: front page uid; for sections: chapter uid; for sub-sections: section uid |
| `identity` | Node type: 101, 102, or 103 |
| `parent_identity` | Identity of the parent node |

**Response `200`**
```json
{
  "new_node": {
    "uid": 11,
    "parent_id": 10,
    "title": "Introduction to PostgreSQL",
    "content": "<basic> ...",
    "images": "[]",
    "identity": 101,
    "page_id": 10
  },
  "update_node": { "uid": 13, "parent_id": 11 }
}
```

---

#### `POST /book/upload`

**Upload an entire book from a ZIP archive in one request.**

See [book-upload.md](book-upload.md) for the complete guide.

**Request** — `multipart/form-data` with a single field containing a `.zip` file.  
**Body limit** — 50 MB.

**Response `200`**
```json
{
  "doc_id": 7,
  "title": "My Book",
  "chapters": 4
}
```

---

#### `POST /book/edit/main`

Edit the front-page node (identity 100) and the `books` summary row.

**Request body**
```json
{
  "doc_id": 5,
  "uid": 10,
  "title": "Updated Book Title",
  "content": "<basic> Updated overview...",
  "images": []
}
```

**Response `200`**
```json
{
  "doc_id": 5,
  "uid": 10,
  "title": "Updated Book Title",
  "content": "<basic> ..."
}
```

---

#### `POST /book/edit/node`

Edit any child node (chapter, section, or sub-section).

**Request body**
```json
{
  "uid": 11,
  "doc_id": 5,
  "title": "Updated Chapter Title",
  "content": "<basic> Updated content...",
  "identity": 101,
  "images": []
}
```

**Response `200`** — echoes the request body.

---

#### `POST /book/delete`

Soft-delete an entire book.

**Request body**
```json
{ "doc_id": 5 }
```

**Response `200`**
```json
{ "data": "book deleted" }
```

---

#### `POST /book/delete/node`

Soft-delete a chapter (101) or section (102) node and cascade-delete its children. Re-links the chain if a successor exists.

**Request body**
```json
{
  "identity": 101,
  "delete_id": 11,
  "parent_id": 10
}
```

| Field | Description |
|-------|-------------|
| `identity` | Identity of the node being deleted (101 or 102) |
| `delete_id` | UID of the node to delete |
| `parent_id` | UID that should become the new `parent_id` of any successor node |

**Cascade behaviour**

- Deleting a section (102) also deletes all its sub-sections (103, via `page_id`).
- Deleting a chapter (101) also deletes all its sections and their sub-sections.

**Response `200`**
```json
{
  "delete_nodes": [11, 20, 21],
  "update_node": { "uid": 12, "parent_id": 10 },
  "rows": 3
}
```

---

## File

### Public — Read

#### `GET /file/blog/:uid/:size/:filename`

Serve a blog image from S3.

| Param | Description |
|-------|-------------|
| `uid` | Document (blog) ID |
| `size` | `340`, `720`, or `1420` (pixel width) |
| `filename` | UUID filename with extension |

**Response** — binary image with appropriate `Content-Type`.

---

#### `GET /file/book/:uid/:size/:filename`

Same as above but for book images.

---

#### `GET /file/tmp/:uid/:size/:filename`

Serve a temporarily uploaded image (before it is moved to its final bucket).

---

### Protected — Write

#### `POST /file/upload`

Upload and crop an image. Returns a filename to include in subsequent create/edit requests.

**Request** — `multipart/form-data` with two fields in order:

1. **metadata** (JSON string)
```json
{
  "oriImgMd": { "width": 1920, "height": 1080 },
  "cropImgMd": { "x": 100, "y": 50, "width": 800, "height": 600 }
}
```

2. **image file** — JPEG, PNG, WebP, or GIF; max 12 MB.

The server crops the image using `cropImgMd` and stores three sizes (340 px, 720 px, 1420 px wide) in the `tmp` S3 bucket under `{user_id}/{size}-{filename}`.

When the document is created/updated, images are moved from `tmp` to the `blog` or `book` bucket automatically.

**Response `200`**
```json
{ "name": "550e8400-e29b-41d4-a716-446655440000.jpg" }
```

---

## User

### Protected

#### `POST /user/:user_id/subscribe`

Follow (subscribe to) a user.

**Path params:** `user_id` — the user to follow.

**Response `200`**
```json
{ "message": "Created" }
```
or `{ "message": "Already subscribed." }` if already following.

---

#### `POST /user/:user_id/un_subscribe`

Unfollow a user.

**Response `200`**
```json
{ "message": "User unfollowed." }
```

---

#### `GET /user/get_subscribed_users`

Returns an array of user IDs that the authenticated user follows.

**Response `200`**
```json
[3, 7, 12]
```

---

## Content Format

All `content` fields use a renderer prefix:

| Prefix | Renderer |
|--------|----------|
| `<basic>` | GitHub-flavoured Markdown + syntax highlighting |
| `<math>` | Markdown + KaTeX (LaTeX math) |

Example: `"<basic> # Heading\n\nSome **bold** text."`.

---

## Error Responses

| Status | Meaning |
|--------|---------|
| `400 Bad Request` | Malformed input or business-rule violation |
| `401 Unauthorized` | Missing or invalid `access_token` cookie |
| `404 Not Found` | Resource does not exist |
| `500 Internal Server Error` | Unexpected server-side failure |

Error bodies are plain-text strings describing the problem.
