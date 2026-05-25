# Book Upload — ZIP Import Guide

**Endpoint:** `POST /book/upload`  
**Auth required:** yes (`access_token` cookie)  
**Body limit:** 50 MB  
**Content-Type:** `multipart/form-data`

Upload a full book — cover, chapters, sections, and sub-sections — in a single request by providing a ZIP archive that follows the folder structure below.

---

## ZIP Folder Structure

```
my-book.zip
└── my-book/                          ← optional root wrapper (auto-stripped)
    ├── cover.md                      ← book cover page  (identity 100)
    ├── 01-introduction/              ← chapter 1        (identity 101)
    │   ├── _index.md                 ← chapter intro    (optional)
    │   ├── 01-what-is-postgres/      ← section 1.1      (identity 102)
    │   │   ├── _index.md             ← section intro    (optional)
    │   │   ├── 01-overview.md        ← sub-section      (identity 103)
    │   │   └── 02-history.md         ← sub-section      (identity 103)
    │   └── 02-installation/          ← section 1.2      (identity 102)
    │       └── 01-install-linux.md   ← sub-section      (identity 103)
    └── 02-data-types/                ← chapter 2        (identity 101)
        └── 01-scalars/               ← section 2.1      (identity 102)
            ├── 01-integers.md        ← sub-section      (identity 103)
            └── 02-text.md            ← sub-section      (identity 103)
```

### Rules

| Level | What it is | Identity | File/Folder |
|-------|------------|----------|-------------|
| Root `.md` | Book cover | 100 | `cover.md`, `index.md`, or `README.md` |
| Depth-1 folder | Chapter | 101 | Any folder; optional `_index.md` inside |
| Depth-2 folder | Section | 102 | Sub-folder of a chapter; optional `_index.md` inside |
| Depth-2 `.md` file | Sub-section | 103 | `.md` files inside a section folder |

- The ZIP may or may not have a single root wrapper folder — it is stripped automatically.
- Files and folders are **ordered by numeric prefix** (`01-`, `02-`, …). Items without a prefix are sorted last.
- Files at depths other than those listed (e.g., `.md` files directly in a chapter folder other than `_index.md`) are silently ignored.
- Only `.md` files are processed; all other files are ignored.

---

## Naming Conventions

**Numeric prefix** sets the sort order and is stripped from the title:
```
01-getting-started  →  "Getting Started"
03-advanced-topics  →  "Advanced Topics"
```

**Title extraction** — if the first line of a file is a Markdown H1, it becomes the node title and is stripped from the body:
```markdown
# My Custom Title

Body content starts here...
```
→ title: `"My Custom Title"`, content: `"<basic> Body content starts here..."`

If no H1 is present, the title comes from the filename/folder name.

---

## cover.md

The cover file sets the book title and the front-page content. Name it `cover.md`, `index.md`, or `README.md`.

```markdown
# The Complete Guide to PostgreSQL

A comprehensive reference for developers and database administrators.

This book covers installation, schema design, querying, and performance tuning.
```

---

## Chapter `_index.md`

Optional. Provides the intro content shown when a user opens a chapter.

```markdown
# Introduction

This chapter introduces PostgreSQL and its history.
```

If omitted, the chapter node is created with empty content.

---

## Section `_index.md`

Optional. Provides the intro content shown when a user opens a section.

```markdown
# What is PostgreSQL?

PostgreSQL is an open-source relational database...
```

---

## Sub-section Files

Regular `.md` files inside a section folder become sub-section nodes (identity 103).

```markdown
# Installing on Linux

Install PostgreSQL via your package manager:

```bash
sudo apt install postgresql
```
```

---

## Request

```bash
curl -s -X POST http://localhost:8003/book/upload \
  -b cookies.txt \
  -F "file=@my-book.zip"
```

Using HTTPie:
```bash
http --session=./session.json POST http://localhost:8003/book/upload \
  file@my-book.zip
```

---

## Response

**`200 OK`**
```json
{
  "doc_id": 7,
  "title": "The Complete Guide to PostgreSQL",
  "chapters": 4
}
```

| Field | Description |
|-------|-------------|
| `doc_id` | The newly created book's document ID; use this for all subsequent API calls |
| `title` | The title extracted from `cover.md` |
| `chapters` | Number of top-level chapters created |

---

## Error Responses

| Status | Message | Cause |
|--------|---------|-------|
| `400` | `No file field in multipart body` | Multipart body is empty or field is missing |
| `400` | `Invalid ZIP file: …` | File is not a valid ZIP archive |
| `400` | `ZIP contains no .md files` | Archive has no processable content |
| `400` | `ZIP must contain a cover.md / index.md / README.md at the root level` | No cover file found after stripping the root prefix |
| `400` | `Cannot read <path>: …` | A file inside the ZIP could not be decoded as UTF-8 |
| `401` | — | Missing or expired `access_token` cookie |
| `500` | — | Database or server-side error |

---

## Database Layout After Upload

```
books (summary row)
└── book identity=100  (front page / cover)
    ├── book identity=101  chapter 1   parent_id → front page uid
    │   ├── book identity=102  section 1.1   page_id → chapter1 uid
    │   │   ├── book identity=103  sub 1.1.1   page_id → section1.1 uid
    │   │   └── book identity=103  sub 1.1.2   page_id → section1.1 uid
    │   └── book identity=102  section 1.2   page_id → chapter1 uid
    │       └── book identity=103  sub 1.2.1   page_id → section1.2 uid
    └── book identity=101  chapter 2   parent_id → chapter1 uid
        └── …
```

Nodes within each level form a **singly linked list** via `parent_id`:
- Chapter 1 → Chapter 2 → Chapter 3 (each chapter's `parent_id` = previous chapter uid)
- Section 1.1 → Section 1.2 (each section's `parent_id` = previous section uid within the same chapter)
- Sub-sections similarly chained within their section

`page_id` is the "owner page" used for bulk retrieval:
- Chapter `page_id` = front-page uid
- Section `page_id` = chapter uid
- Sub-section `page_id` = section uid

---

## Creating a ZIP from a Local Folder

```bash
# On macOS / Linux
cd /path/to/my-book
zip -r ../my-book.zip .

# Or with a root wrapper folder
cd /path/to/parent
zip -r my-book.zip my-book/
```

Both forms are supported — the root wrapper is auto-detected and stripped.
