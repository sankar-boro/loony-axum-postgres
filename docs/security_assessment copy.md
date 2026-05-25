# Security Assessment Report — Loony-Book

**Services Reviewed:** `loony-auth` (Rust/Axum auth service), `loony-book-backend` (Rust/Axum API), `loony-book-frontend` (React/TypeScript SPA)
**Date:** 2026-05-23
**Methodology:** White-box source code review, OWASP Top 10

---

## Executive Summary

The application has a solid security foundation in several areas (Argon2 password hashing,
parameterised SQL, HttpOnly/Secure JWT cookies, TLS enforcement, username enumeration
prevention). However, **three critical-severity vulnerabilities** were found that would allow any
authenticated user to delete or modify any other user's content, and to read arbitrary files from
the server filesystem.

---

## Findings

---

### [CRITICAL-1] Insecure Direct Object Reference (IDOR) — No Ownership Verification on Delete / Edit

**Severity:** Critical
**Affected files:**
- `src/blog/delete.rs` — `delete_blog`, `delete_blog_node`
- `src/book/delete.rs` — `delete_book`, `delete_book_node`
- `src/blog/mod.rs` — `edit_blog`, `edit_blog_node`, `append_blog_node`
- `src/book/edit.rs` — `edit_book`, `edit_book_node`

**Description:**

**Delete handlers** do not extract the user from the session at all:

```rust
// blog/delete.rs:66 — NO Session parameter
pub async fn delete_blog(
    State(pool): State<AppState>,
    Json(body): Json<DeleteBlog>,
```

**Edit handlers** extract `user_id` from the session but then discard it — the WHERE clause is
keyed on the client-supplied `doc_id`/`uid`, not filtered by `user_id`:

```rust
// blog/mod.rs:139 — user_id fetched, then never used in the query
let user_id = session.get_user_id().await?;
...
transaction.execute(&state_1, &[&body.title, &body.content, &images, &body.doc_id]).await?;
```

**Exploitation:**
1. Attacker logs in as User A, obtains a valid session cookie.
2. Attacker discovers any `doc_id` (IDs are sequential integers, trivially enumerable).
3. `POST /blog/delete` with `{"doc_id": 42}` — deletes User B's blog.
4. `POST /blog/edit/main` with any `doc_id` / `uid` — overwrites User B's content.

**Impact:** Any authenticated user can permanently delete or modify any other user's blogs and books.

**Fix:**
```rust
// Add user_id to the WHERE clause on all mutations:
"UPDATE blogs SET deleted_at=$1 WHERE uid=$2 AND user_id=$3"
// params: [&current_time, &body.doc_id, &user_id]

// For edit_blog:
"UPDATE blogs SET title=$1, content=$2, images=$3 WHERE uid=$4 AND user_id=$5"
```

---

### [CRITICAL-2] Path Traversal in File-Serving Endpoints

**Severity:** Critical
**Affected file:** `src/file/mod.rs` — `get_blog_file`, `get_book_file`, `get_tmp_file` (lines 110–151)

**Description:**

Three endpoints accept `size` and `filename` as raw string path segments and concatenate them
directly into a filesystem path with no sanitisation:

```rust
// file/mod.rs:110
pub async fn get_blog_file(
    State(state): State<AppState>,
    AxumPath((uid, size, filename)): AxumPath<(i32, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let file_path = format!("{}/{}/{}-{}", &state.get_blog_path(), uid, size, filename);
    let f = std::fs::read(&file_path)?;
```

`uid` is `i32` (safe), but `size` and `filename` are unsanitised `String`.

**Exploitation (URL-decoded by Axum):**
```
GET /file/blog/1/../../etc/1-passwd
```
Constructs: `{blog_path}/1/../../etc/1-passwd` — resolves to `/etc/passwd` depending on
`blog_path` depth.

Or with percent-encoding to bypass naive checks:
```
GET /file/tmp/1/%2e%2e/%2e%2e/etc/1-shadow
```

**Impact:** Arbitrary file read on the server — private keys, env files, `/proc/self/environ`,
database credentials.

**Fix:**
```rust
use std::path::Path;

fn safe_filename(s: &str) -> bool {
    !s.contains('/') && !s.contains('\\') && !s.contains("..")
}

// In handler, validate both parameters:
if !safe_filename(&size) || !safe_filename(&filename) {
    return Err(AppError::BadRequest("Invalid path component".into()));
}

// Also canonicalize and verify the resolved path stays within the base dir:
let resolved = std::fs::canonicalize(&file_path)?;
if !resolved.starts_with(&base_path) {
    return Err(AppError::BadRequest("Path traversal detected".into()));
}
```

---

### [CRITICAL-3] Panic / DoS via Crafted Multipart Upload

**Severity:** Critical (any authenticated user can trigger)
**Affected file:** `src/file/mod.rs` (lines 37–54)

**Description:**

Multiple `.unwrap()` calls in the upload handler panic on attacker-controlled input:

```rust
// file/mod.rs:39
let metadata_str = std::str::from_utf8(&metadata_bytes).unwrap();  // panics on non-UTF-8
let img_metadata: ImageMetadata = serde_json::from_str(&metadata_str).unwrap(); // panics on bad JSON

// file/mod.rs:42
let filename = &img_field.file_name().unwrap().to_string(); // panics if no filename field

// file/mod.rs:52
let format = ImageFormat::from_extension(extension);
let dynamic_image = image::load(cursor, format.unwrap())?; // panics on unknown extension
```

There is also no file-type allowlist — any extension (`.php`, `.sh`, `.exe`) passes through
until the panic.

**Exploitation:**
```bash
curl -X POST /file/upload \
  -F 'metadata={"oriImgMd":{"width":100,"height":100},"cropImgMd":{"x":0,"y":0,"width":50,"height":50}}' \
  -F 'file=@malware.php;filename=shell.php'
# ImageFormat::from_extension("php") returns None → format.unwrap() panics
```

**Impact:** Server instability; repeated requests saturate worker threads with panics.

**Fix:**
```rust
// Replace all .unwrap() with proper error propagation:
let metadata_str = std::str::from_utf8(&metadata_bytes)
    .map_err(|_| AppError::BadRequest("Invalid UTF-8 in metadata".into()))?;
let img_metadata: ImageMetadata = serde_json::from_str(&metadata_str)
    .map_err(|_| AppError::BadRequest("Invalid metadata JSON".into()))?;
let filename = img_field.file_name()
    .ok_or_else(|| AppError::BadRequest("Missing filename".into()))?
    .to_string();

// Allowlist extensions before calling image::load:
let allowed = ["jpg", "jpeg", "png", "webp", "gif"];
if !allowed.contains(&extension.to_lowercase().as_str()) {
    return Err(AppError::BadRequest("Unsupported file type".into()));
}
let format = ImageFormat::from_extension(extension)
    .ok_or_else(|| AppError::BadRequest("Unknown image format".into()))?;
```

---

### [HIGH-1] Internal Error Messages Leaked to Clients

**Severity:** High
**Affected file:** `src/error.rs` (lines 83–95)

**Description:**

The backend's `IntoResponse` for `AppError::InternalServerError` directly returns the raw error
string to the client:

```rust
// error.rs:89
AppError::InternalServerError(e) => e,  // raw message sent as response body
...
(StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
```

This exposes database error messages, file paths, tokio-postgres internals (e.g., `column "x" of
relation "blogs" does not exist`), and connection strings.

The `loony-auth` service already has the correct pattern:
```rust
// loony-auth/src/error/mod.rs — correct
AppError::InternalServerError(e) => {
    tracing::error!(error = %e, "internal server error");
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
}
```

**Fix:** Apply the same redaction pattern to `src/error.rs`:
```rust
AppError::InternalServerError(e) => {
    tracing::error!(error = %e, "internal server error");
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
}
```

---

### [HIGH-2] No File MIME Validation — Polyglot / Malicious Upload

**Severity:** High
**Affected file:** `src/file/mod.rs` (lines 44–54)

**Description:**

File format is determined solely from the client-supplied filename extension, not from actual
file content (magic bytes). An attacker can upload a file with a valid image extension (`.jpg`)
that contains malicious content — polyglots, EICAR, or embedded scripts.

```rust
let extension = Path::new(&filename).extension()...;
let format = ImageFormat::from_extension(extension);
let dynamic_image = image::load(cursor, format.unwrap())?;
```

Adversarial image files (e.g., crafted to exploit decoder bugs) are not filtered before being
fed into `image::load`.

**Fix:** Check magic bytes before processing:
```rust
fn is_image(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\xFF\xD8\xFF")          // JPEG
    || bytes.starts_with(b"\x89PNG\r\n\x1a\n")  // PNG
    || bytes.starts_with(b"GIF87a")
    || bytes.starts_with(b"GIF89a")
    || bytes.starts_with(b"RIFF")               // WebP
}

if !is_image(img_bytes) {
    return Err(AppError::BadRequest("File content does not match an image".into()));
}
```

---

### [MEDIUM-1] User/Email Enumeration via Password Reset

**Severity:** Medium
**Affected file:** `loony-auth/src/models/reset_password.rs` (lines 41–54)

**Description:**

The reset flow returns **different error messages** depending on whether the session token is
invalid vs. whether the email address exists:

```rust
// "Reset link is invalid or has expired."
None => return Err(email_link_expired()),

// "User not found."  ← reveals email does not exist
if row.is_none() {
    return Err(user_not_found());
}
```

Combined with the unauthenticated, unrate-limited `/mail` endpoint, an attacker can probe email
existence.

**Fix:** Return the same error regardless of which check failed:
```rust
if row.is_none() {
    return Err(email_link_expired()); // not user_not_found()
}
```

---

### [MEDIUM-2] No Rate Limiting on Authentication Endpoints

**Severity:** Medium
**Affected file:** `loony-auth/src/routes/mod.rs`

**Description:**

The `/login`, `/signup`, `/mail`, and `/resetPassword` endpoints have no rate limiting or account
lockout. This enables brute-force password attacks and password-reset token enumeration. The test
script (`scripts/test_security.sh` section 8) explicitly documents this as a known gap with a
`WARN` result.

**Fix:**
```rust
use axum_governor::{GovernorConfigBuilder, GovernorLayer};

let login_governor = GovernorConfigBuilder::default()
    .per_second(3)
    .burst_size(5)
    .finish().unwrap();

Router::new()
    .route("/login", post(login))
    .layer(GovernorLayer { config: Arc::new(login_governor) })
```
Apply separate governors to `/login`, `/signup`, and `/mail`.

---

### [MEDIUM-3] Missing Security Response Headers

**Severity:** Medium
**Affected:** Both backend services

**Description:**

Neither service sets standard defensive HTTP response headers:

| Header | Protection |
|---|---|
| `Content-Security-Policy` | XSS / resource injection |
| `X-Content-Type-Options: nosniff` | MIME-sniffing attacks |
| `X-Frame-Options: DENY` | Clickjacking |
| `Strict-Transport-Security` | HTTPS downgrade |
| `Referrer-Policy` | Referrer leakage |

**Fix:** Add a header-injection middleware layer:
```rust
use tower_http::set_header::SetResponseHeaderLayer;

router.layer(SetResponseHeaderLayer::overriding(
    header::HeaderName::from_static("x-content-type-options"),
    HeaderValue::from_static("nosniff"),
))
.layer(SetResponseHeaderLayer::overriding(
    header::HeaderName::from_static("x-frame-options"),
    HeaderValue::from_static("DENY"),
))
.layer(SetResponseHeaderLayer::overriding(
    header::HeaderName::from_static("referrer-policy"),
    HeaderValue::from_static("strict-origin-when-cross-origin"),
))
```

---

### [MEDIUM-4] `SameSite=None` Cookies Without CSRF Protection

**Severity:** Medium
**Affected files:**
- `loony-auth/src/utils/cookie.rs` (line 55)
- `src/connections/session.rs` (line 12)

**Description:**

All cookies use `SameSite=None`, meaning browsers include them in **all** cross-origin requests:

```rust
CookieBuilder::new(name, value)
    .same_site(cookie::SameSite::None)
    .secure(true)
    .http_only(true)
```

`HttpOnly` prevents JavaScript token theft, but CSRF remains possible: a malicious page can
trigger state-changing POST requests that include the victim's cookies. CORS only limits reading
the response — it does not block the request from executing.

**Fix:** If the frontend and API share the same registered site, switch to `SameSite=Lax` or
`SameSite=Strict`. If cross-origin is required, implement a CSRF double-submit cookie or
synchroniser token.

---

### [MEDIUM-5] SQL Parameter Mismatch in `get_subscribed_users`

**Severity:** Medium (runtime error with information disclosure)
**Affected file:** `src/user/mod.rs` (lines 21–26)

**Description:**

The query has one placeholder but receives two parameters:

```rust
let rows = conn.query(
    "SELECT subscribed_id FROM subscription where user_id=$1",
    &[&auth_user_id, &user_id],  // ← two params, one placeholder
).await?;
```

`tokio_postgres` returns an error that propagates as `AppError::InternalServerError`, which
(per HIGH-1) leaks the raw postgres error message to the client. The endpoint never functions
correctly.

**Fix:**
```rust
"SELECT subscribed_id FROM subscription where user_id=$1",
&[&auth_user_id],
```

---

### [MEDIUM-6] CORS Misconfiguration — Response Header Listed as Allowed Request Header

**Severity:** Low-Medium
**Affected file:** `src/connections/cors.rs` (line 26)

**Description:**

`ACCESS_CONTROL_ALLOW_ORIGIN` is a CORS response header, not a request header. Listing it in
`allow_headers` is incorrect and reflects a misunderstanding of the CORS model:

```rust
.allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN])
```

**Fix:** Remove `ACCESS_CONTROL_ALLOW_ORIGIN` from the `allow_headers` list.

---

### [LOW-1] `validate_nbf` Not Set in Backend JWT Decoder

**Severity:** Low
**Affected file:** `src/auth/mod.rs` (lines 9–13)

**Description:**

The backend JWT validator omits `validate_nbf`, while the auth service sets it to `true`:

```rust
// src/auth/mod.rs — nbf NOT validated (defaults to false)
let mut validation = Validation::new(Algorithm::HS256);
validation.validate_exp = true;

// loony-auth/src/utils/auth.rs — nbf IS validated
validation.validate_nbf = true;
```

A token with a future `nbf` claim could be accepted before its intended validity window.

**Fix:**
```rust
validation.validate_nbf = true;
```

---

### [LOW-2] Unsubstituted `{{YEAR}}` Placeholder in Password Reset Email

**Severity:** Low
**Affected file:** `loony-auth/src/utils/data.rs` (line 37)

**Description:**

The email template includes `© {{YEAR}} Your Company. All rights reserved.` — the `{{YEAR}}`
placeholder is never replaced (only `{{RESET_LINK}}` is substituted), so users receive broken
HTML that leaks the template marker.

**Fix:**
```rust
let body = BODY
    .replace("{{RESET_LINK}}", &format!("/resetPassword/{token_id}"))
    .replace("{{YEAR}}", &chrono::Utc::now().format("%Y").to_string());
```

---

### [LOW-3] TLS Certificate Path Hardcoded in Auth Server

**Severity:** Low
**Affected file:** `loony-auth/src/axum_server.rs` (lines 49–52)

**Description:**

Certificate paths are hardcoded relative strings rather than read from configuration:

```rust
RustlsConfig::from_pem_file(
    ".local/localhost.pem",
    ".local/localhost-key.pem",
)
```

The `config.rs` already defines a `CertsConfig` struct with `key_path` / `cert_path` fields, but
they are unused here. This prevents certificate rotation without a code change.

**Fix:** Read cert paths from the loaded `Config` struct.

---

## Areas That Appear Secure

- **SQL Injection:** All database queries use parameterised statements (`$1`, `$2`, …) via
  `tokio_postgres`. No string-interpolated SQL found anywhere in the codebase.

- **Password Hashing:** Argon2id is correctly used with `OsRng`-generated salts. The migration
  path from legacy bcrypt hashes is handled safely in `loony-auth/src/models/login.rs`.

- **JWT Algorithm Confusion (`alg:none`):** `Validation` specifies `Algorithm::HS256` explicitly.
  The test suite (`scripts/test_security.sh` section 4) confirms `alg:none` tokens are rejected.

- **Username Enumeration on Login:** Both "user not found" and "wrong password" return identical
  responses (`{"message":"Invalid credentials."}`). Verified in `loony-auth/src/error/message.rs`.

- **Password Reset Replay:** `session.remove()` is called on successful reset, invalidating the
  token. Verified in `loony-auth/src/models/reset_password.rs:65`.

- **Body Size Limit:** A 12 MB request body limit is enforced via `RequestBodyLimitLayer` on
  both services.

- **Cookie Security Flags:** All cookies are `Secure` and `HttpOnly` with a 30-minute access
  token lifetime.

- **XSS (Stored):** The frontend renders user content through `react-markdown`, which sanitises
  HTML by default. No `dangerouslySetInnerHTML` calls found in the codebase.

---

## Prioritised Remediation Roadmap

| Priority | ID | Issue | Effort |
|---|---|---|---|
| 1 | CRITICAL-1 | Add `AND user_id=$N` to all mutating SQL queries | Low |
| 2 | CRITICAL-2 | Sanitise `size` and `filename` path params in file endpoints | Low |
| 3 | CRITICAL-3 | Replace `.unwrap()` in upload handler; add extension allowlist | Low |
| 4 | HIGH-1 | Redact internal errors in `src/error.rs` | Low |
| 5 | HIGH-2 | Add magic-byte MIME validation to file upload | Low |
| 6 | MEDIUM-1 | Return same error for all reset-password failure modes | Low |
| 7 | MEDIUM-2 | Add rate limiting to `/login`, `/signup`, `/mail` | Medium |
| 8 | MEDIUM-3 | Add security response headers via middleware layer | Low |
| 9 | MEDIUM-4 | Upgrade `SameSite` cookie policy or add CSRF tokens | Medium |
| 10 | MEDIUM-5 | Fix SQL parameter count in `get_subscribed_users` | Low |
