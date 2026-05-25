# loony-s3 Integration Guide

Everything you need to upload and retrieve objects through loony-s3.

---

## Credentials

loony-s3 has no concept of access-key-id / secret-access-key pairs like AWS S3.
Authentication is done with a **JWT signed by a shared secret**.

| What | Where it lives | Description |
|---|---|---|
| **JWT secret** | `JWT_SECRET` in loony-s3's env | Used to verify every incoming token |
| **Caller identity** | `sub` field in the JWT payload | Identifies who owns/accesses resources |
| **Caller name** | `name` field in the JWT payload | Arbitrary label, used only for logging |

There is no region, no access key, and no account ID.

### Generating a token

Tokens are HS256 JWTs. The minimum payload is:

```json
{
  "sub": "<your-user-id>",
  "name": "<any-label>",
  "exp": <unix-timestamp>
}
```

**In Rust (loony-book-backend):**
```rust
// src/file/s3_client.rs — make_jwt()
let claims = Claims { sub: user_id.to_string(), name: "loony-api".to_string(), exp };
encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes()))
```

**In JavaScript:**
```js
const jwt = require('jsonwebtoken');
const token = jwt.sign(
  { sub: "42", name: "my-service" },
  process.env.JWT_SECRET,
  { expiresIn: "1h" }
);
```

**With curl (quick test, requires python3):**
```bash
JWT_SECRET="your-secret-here"
TOKEN=$(python3 -c "
import hmac, hashlib, base64, json, time
s = '$JWT_SECRET'
h = base64.urlsafe_b64encode(json.dumps({'alg':'HS256','typ':'JWT'}).encode()).rstrip(b'=').decode()
p = base64.urlsafe_b64encode(json.dumps({'sub':'1','name':'test','exp':int(time.time())+3600}).encode()).rstrip(b'=').decode()
sig = base64.urlsafe_b64encode(hmac.new(s.encode(),f'{h}.{p}'.encode(),hashlib.sha256).digest()).rstrip(b'=').decode()
print(f'{h}.{p}.{sig}')
")
```

Token goes in every request as:
```
Authorization: Bearer <token>
```

---

## Bucket

A bucket is a named container for objects. You must create a bucket before uploading to it.

### Bucket fields

| Field | Type | Required | Default | Notes |
|---|---|---|---|---|
| `name` | string | yes | — | 3–63 chars, lowercase alphanumeric, hyphens, dots. No `..`, `.-`, `-.`. |
| `acl` | `"private"` \| `"public-read"` \| `"public-read-write"` | no | `"private"` | Controls who can read/write. See ACL table below. |
| `region` | string | no | `"us-east-1"` | Stored as metadata only — no actual routing effect. |
| `versioning` | boolean | no | `false` | Keep previous versions of overwritten objects. |
| `metadata` | `Record<string, string>` | no | `{}` | Arbitrary key-value pairs. |

### Bucket ACL

| Value | Who can read | Who can write |
|---|---|---|
| `private` | owner only | owner only |
| `public-read` | anyone (no token needed) | owner only |
| `public-read-write` | anyone | any authenticated user |

For loony-book-backend the buckets `tmp`, `blog`, and `book` are created with
`public-read-write` so any authenticated user can upload to them.

### Create a bucket

```
POST /buckets
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "my-bucket",
  "acl": "public-read-write",
  "region": "us-east-1"
}
```

Response `201`:
```json
{
  "bucket": {
    "id": "uuid",
    "name": "my-bucket",
    "ownerId": "<sub from token>",
    "acl": "public-read-write",
    "region": "us-east-1",
    "versioning": false,
    "metadata": {},
    "createdAt": "...",
    "updatedAt": "..."
  }
}
```

Returns `409` if the bucket name already exists.

### Update a bucket (e.g. change ACL)

Only the bucket **owner** can update it.

```
PATCH /buckets/:name
Authorization: Bearer <owner-token>
Content-Type: application/json

{ "acl": "public-read-write" }
```

---

## Uploading an object

```
PUT /:bucket/:key
Authorization: Bearer <token>
Content-Type: <mime-type>
x-acl: public-read          (optional, default: private)
x-ttl-seconds: 3600         (optional, object auto-deletes after N seconds)
Content-Length: <bytes>     (optional but recommended)
```

The body is the raw file bytes. The key is the object path within the bucket,
e.g. `1/340-photo.png`.

loony-book-backend uses the key pattern `{user_id}/{size}-{filename}`:

```
PUT /tmp/1/340-abc123.png
Authorization: Bearer <token-with-sub="1">
Content-Type: image/png
x-acl: public-read
<raw image bytes>
```

Response `200`:
```json
{
  "key": "1/340-abc123.png",
  "size": 48210,
  "mime_type": "image/png",
  "etag": "d41d8cd98f00b204e9800998ecf8427e",
  "version_id": "abc...",
  "is_latest": true,
  "acl": "public-read",
  "metadata": {},
  "created_at": "...",
  "updated_at": "...",
  "expires_at": null
}
```

Write is rejected with `403 ACCESS_DENIED` if:
- bucket ACL is `private` or `public-read` AND
- the token's `sub` is not the bucket's `ownerId`

---

## Downloading an object

Public-read objects need no token. Private objects require the owner's token.

```
GET /:bucket/:key
```

Supports `Range` header for partial content (returns `206`).

loony-book-backend serves files through its own proxy at:
```
GET /file/tmp/:uid/:size/:filename   → GET /tmp/{uid}/{size}-{filename}
GET /file/blog/:uid/:size/:filename  → GET /blog/{uid}/{size}-{filename}
GET /file/book/:uid/:size/:filename  → GET /book/{uid}/{size}-{filename}
```

---

## Object metadata headers

| Header | Direction | Description |
|---|---|---|
| `x-acl` | upload | Object ACL: `"private"` or `"public-read"` |
| `x-ttl-seconds` | upload | Auto-expire in N seconds |
| `x-amz-meta-*` | upload | Custom metadata key-value (stored, returned on download) |
| `ETag` | response | MD5 hex of the object content |
| `x-version-id` | response | Version UUID |

---

## Required environment variables

### loony-s3 (`~/.envs/s3/back.env`)

| Variable | Required | Default | Notes |
|---|---|---|---|
| `JWT_SECRET` | yes | `dev-secret-change-in-production` | Must match `S3_JWT_SECRET` in the backend |
| `PORT` | no | `3000` | Port loony-s3 listens on |
| `STORAGE_BACKEND` | no | `local` | `local` or `nfs` |
| `STORAGE_ROOT` | no | `/tmp/loony-s3/data` | Where files are stored on disk |
| `DB_PATH` | no | `/tmp/loony-s3/metadata.db` | SQLite metadata database path |
| `DB_BACKEND` | no | `sqlite` | `sqlite` or `postgres` |
| `DATABASE_URL` | if postgres | — | Postgres connection string |
| `PRESIGNED_SECRET` | no | `dev-presigned-secret` | Secret for presigned URL HMAC |
| `MAX_OBJECT_SIZE_BYTES` | no | `5368709120` (5 GB) | Per-object size limit |

### loony-book-backend (`~/.envs/book/back.env`)

| Variable | Required | Notes |
|---|---|---|
| `S3_URL` | yes | Base URL of loony-s3, e.g. `http://localhost:8006` |
| `S3_JWT_SECRET` | yes | Must match `JWT_SECRET` in loony-s3's env |

---

## Complete upload flow (what loony-book-backend does)

```
1. POST /file/upload  (multipart/form-data with metadata + image)
        │
        ▼
2. require_auth middleware
   → reads access_token cookie
   → decodes JWT with SECRET_KEY + AUTH_APP_NAME
   → injects UserId(user_id) into request extensions
        │
        ▼
3. upload_file handler
   → reads metadata field (oriImgMd, cropImgMd)
   → reads file field, validates magic bytes and extension
   → decodes and crops the image
   → produces three sizes: 340px, 720px, 1420px
        │
        ▼
4. S3Client::put("tmp", "{user_id}/{size}-{uuid}.ext", bytes, mime, user_id)
   → signs a JWT: { sub: user_id, name: "loony-api", exp: now+3600 }
   → PUT http://localhost:8006/tmp/{user_id}/{size}-{uuid}.ext
       Authorization: Bearer <signed-jwt>
       x-acl: public-read
       Content-Type: image/png
        │
        ▼
5. Returns { "name": "{uuid}.ext" }

6. When the document is saved (create_book, etc.):
   move_images_to_s3 copies each size variant:
     tmp/{user_id}/{size}-{name}  →  book/{doc_id}/{size}-{name}
   then deletes the source from tmp.
```

---

## Quick reference: curl examples

```bash
BASE=http://localhost:8006
TOKEN="<your-jwt>"

# Create bucket
curl -X POST $BASE/buckets \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-bucket","acl":"public-read-write"}'

# Upload file (raw body)
curl -X PUT $BASE/my-bucket/images/photo.png \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: image/png" \
  -H "x-acl: public-read" \
  --data-binary @photo.png

# Download file (public-read — no token needed)
curl $BASE/my-bucket/images/photo.png -o downloaded.png

# Delete file
curl -X DELETE $BASE/my-bucket/images/photo.png \
  -H "Authorization: Bearer $TOKEN"

# List your buckets
curl $BASE/buckets \
  -H "Authorization: Bearer $TOKEN"
```
