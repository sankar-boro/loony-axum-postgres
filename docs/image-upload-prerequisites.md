# Image Upload Prerequisites

This document describes every step that must be complete before `POST /file/upload` will succeed.

---

## 1. All three services must be running

| Service | Default port | Start command |
|---|---|---|
| **loony-s3** (`loony-s3-js`) | `8006` | `npm run dev` (or `./dev.sh start s3`) |
| **loony-auth** | `8000` | `cargo run` (or `./dev.sh start auth`) |
| **loony-book-backend** | `8003` | `cargo run` (or `./dev.sh start api`) |

The frontend (`3003`) does not participate in the upload itself, but you need it to trigger the flow.

---

## 2. Environment variables must match across services

Three values must be identical in different env files or the chain breaks.

### `SECRET_KEY` — JWT signing secret (auth ↔ backend)

loony-auth **encodes** the `access_token` JWT with this key.  
loony-book-backend **decodes** it with `AUTH_APP_NAME` + `SECRET_KEY`.

| Service | Env var |
|---|---|
| `~/.envs/auth/back.env` | `SECRET_KEY=<value>` |
| `~/.envs/book/back.env` | `SECRET_KEY=<value>` ← must be the same |

### `APP_NAME` / `AUTH_APP_NAME` — JWT audience and issuer

loony-auth encodes the token with `iss` = `APP_NAME` and `aud` = `[APP_NAME]`.  
loony-book-backend validates with `AUTH_APP_NAME` for both issuer and audience.

| Service | Env var |
|---|---|
| `~/.envs/auth/back.env` | `APP_NAME=<value>` |
| `~/.envs/book/back.env` | `AUTH_APP_NAME=<value>` ← must match the auth `APP_NAME` |

### `S3_JWT_SECRET` — JWT signing secret (backend → loony-s3)

loony-book-backend signs its requests to loony-s3 with this key.  
loony-s3 verifies incoming JWTs with `JWT_SECRET`.

| Service | Env var |
|---|---|
| `~/.envs/book/back.env` | `S3_JWT_SECRET=<value>` |
| `~/.envs/s3/back.env` | `JWT_SECRET=<value>` ← must be the same |

### `S3_URL` — loony-s3 base URL

loony-book-backend must know where loony-s3 is listening.

```
# ~/.envs/book/back.env
S3_URL=http://localhost:8006
```

---

## 3. S3 buckets must exist with `public-read-write` ACL

loony-book-backend calls `ensure_bucket` for `tmp`, `blog`, and `book` at startup.  
It creates each bucket (or patches the ACL if the bucket already exists) to `public-read-write`.

**This requires loony-s3 to be running before loony-book-backend starts.**  
If loony-s3 was not up when loony-api started, restart loony-api after loony-s3 is ready.

You can verify bucket ACLs directly:

```bash
# Requires node or python to produce a JWT — see dev.sh for the correct secret
curl -s -H "Authorization: Bearer <system-jwt>" http://localhost:8006/buckets
```

Expected result: each bucket entry should show `"acl": "public-read-write"`.

---

## 4. The user must be logged in (valid `access_token` cookie)

`POST /file/upload` is behind the `require_auth` middleware.  
The browser must have a valid `access_token` cookie set by loony-auth's `POST /auth/login`.

### Cookie behaviour in development

loony-auth sets `Secure=false` when `APP_ENV != "production"`, so the cookie is sent over plain HTTP.  
If `APP_ENV=production`, the cookie requires HTTPS and **will not be sent** over `http://localhost`.

Verify your auth env has:
```
APP_ENV=development
```

### What `require_auth` checks

1. Parses the `Cookie` request header for `access_token`.
2. Decodes the JWT using `AUTH_APP_NAME` (issuer + audience) and `SECRET_KEY`.
3. Extracts `sub` (the user's numeric id) and injects it as a request extension.
4. Returns `401` if the cookie is missing, expired, or signed with a different secret.

---

## 5. The multipart request must follow the expected field order

`upload_file` reads exactly two multipart fields in order:

1. **Field 1 — `metadata`** (plain text / JSON):
   ```json
   {
     "oriImgMd": { "width": 1920, "height": 1080 },
     "cropImgMd": { "x": 0, "y": 0, "width": 1280, "height": 960 }
   }
   ```
2. **Field 2 — `file`** (binary image data with a filename):
   - Allowed extensions: `jpg`, `jpeg`, `png`, `webp`, `gif`
   - File magic bytes are validated (not just the extension)
   - Maximum size: `12 MB` (enforced by `RequestBodyLimitLayer`)

The frontend (`uploadImage.tsx`) does this via:
```ts
formData.append("metadata", JSON.stringify({ oriImgMd, cropImgMd }))
formData.append("file", imageFile)
```

---

## 6. What happens on a successful upload

1. The image is decoded and cropped using `cropImgMd` coordinates.
2. Three resized variants are produced: `340px`, `720px`, `1420px` wide.
3. All three are uploaded to the `tmp` bucket in loony-s3 under the key pattern:
   ```
   tmp/{user_id}/{size}-{uuid}.{ext}
   ```
4. The response is `{ "name": "{uuid}.{ext}" }`.
5. The frontend previews the image at:
   ```
   GET /file/tmp/{user_id}/340/{uuid}.{ext}
   ```
6. When the document is saved (`create_book`, `append_book_node`, etc.), `move_images_to_s3`
   copies the three variants from `tmp/{user_id}/...` to `book/{doc_id}/...` and deletes the
   originals from `tmp`.

---

## Quick startup checklist

```
[ ] loony-s3 is running on port 8006
[ ] loony-auth is running on port 8000
[ ] S3_JWT_SECRET (book/back.env) == JWT_SECRET (s3/back.env)
[ ] SECRET_KEY (book/back.env) == SECRET_KEY (auth/back.env)
[ ] AUTH_APP_NAME (book/back.env) == APP_NAME (auth/back.env)
[ ] APP_ENV=development in auth/back.env (so Secure cookie is omitted)
[ ] loony-book-backend started AFTER loony-s3 (so ensure_bucket ran successfully)
[ ] User is logged in — browser holds a valid access_token cookie
[ ] Image is > 1420px in its largest dimension (frontend enforces this in onSelectImage)
```
