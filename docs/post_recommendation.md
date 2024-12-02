To create an SQL database that can recommend posts to users based on what they post and search, we need to structure the schema to track users, posts, searches, and interactions. Additionally, we'll need a way to score and recommend posts dynamically based on the collected data.

### **Database Schema**

Here’s a schema design that supports this functionality:

#### **1. Users Table**

Tracks information about the users.

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

#### **2. Posts Table**

Stores the posts made by users.

```sql
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    tags VARCHAR(255), -- Tags for categorization, separated by commas
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

#### **3. Search History Table**

Tracks the searches made by users.

```sql
CREATE TABLE search_history (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    search_query VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

#### **4. Interactions Table**

Logs interactions between users and posts (e.g., likes, comments, or views).

```sql
CREATE TABLE interactions (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id INT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    interaction_type VARCHAR(50) NOT NULL, -- e.g., 'view', 'like', 'comment'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

#### **5. Recommendations Table (Optional, for Cache)**

Pre-computed recommendations (if needed).

```sql
CREATE TABLE recommendations (
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recommended_post_id INT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    score FLOAT NOT NULL, -- Relevance score
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, recommended_post_id)
);
```

---

### **Populating Data**

1. **Insert Sample Users**

```sql
INSERT INTO users (username, email) VALUES
('john_doe', 'john@example.com'),
('jane_smith', 'jane@example.com');
```

2. **Insert Sample Posts**

```sql
INSERT INTO posts (user_id, title, content, tags) VALUES
(1, 'Learning SQL', 'SQL is a powerful language for databases.', 'SQL,Databases'),
(2, 'Rust Programming', 'Rust is great for systems programming.', 'Rust,Programming');
```

3. **Track Searches**

```sql
INSERT INTO search_history (user_id, search_query) VALUES
(1, 'SQL tutorials'),
(2, 'Rust concurrency');
```

4. **Log Interactions**

```sql
INSERT INTO interactions (user_id, post_id, interaction_type) VALUES
(1, 2, 'view'),
(2, 1, 'like');
```

---

### **Recommendation Logic**

To recommend posts, the system can query based on tags, past searches, or interactions.

#### **Example Queries for Recommendations**

1. **Based on Tags (Similar Content)**:

```sql
SELECT p.*
FROM posts p
JOIN posts p2 ON p2.id != p.id
WHERE p2.user_id = 1
  AND p.tags && string_to_array(p2.tags, ',')
LIMIT 5;
```

2. **Based on Search History**:

```sql
SELECT p.*
FROM posts p
JOIN search_history sh ON sh.user_id = 1
WHERE LOWER(p.content) LIKE '%' || LOWER(sh.search_query) || '%'
LIMIT 5;
```

3. **Combine Multiple Factors**:
   For more sophisticated recommendations, calculate a relevance score using a combination of:
   - Tags similarity
   - Content relevance (via search terms)
   - Past interactions (e.g., liked/viewed posts)

Example using a weighted score:

```sql
WITH ranked_posts AS (
    SELECT
        p.id,
        p.title,
        p.content,
        p.tags,
        COUNT(DISTINCT sh.id) * 0.5 + COUNT(DISTINCT i.id) * 0.5 AS score
    FROM posts p
    LEFT JOIN search_history sh ON sh.user_id = 1 AND p.content LIKE '%' || sh.search_query || '%'
    LEFT JOIN interactions i ON i.user_id = 1 AND i.post_id = p.id
    GROUP BY p.id
)
SELECT *
FROM ranked_posts
ORDER BY score DESC
LIMIT 5;
```

---

### **Scaling the Solution**

1. **Indexes**:

   - Add indexes on frequently queried columns like `tags`, `search_query`, and `interaction_type`.

   ```sql
   CREATE INDEX idx_tags ON posts USING gin (string_to_array(tags, ','));
   CREATE INDEX idx_search_query ON search_history (search_query);
   ```

2. **Recommendation Engine**:
   - Use a machine learning model (e.g., collaborative filtering) for better recommendations and store results in the `recommendations` table.

This schema and logic allow flexibility for both content-based and collaborative filtering approaches.
