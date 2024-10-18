# Return Types

# **Blog**

> **_DocumentNode_**

```ts
type DocumentNode = {
  uid: number;
  title: string;
  body: string;
  images: string[];
  user_id: number;
  tags: string[] | null;
  theme: number;
};
```

> Create blog

```ts
type CreateBlog = DocumentNode;
```

> Add Blog Node

```ts
type AddBlogNode = {
  new_node: DocumentNode;
  update_node: {
    uid: number;
    parent_id: null | number;
  };
};
```

> Edit Blog

```ts
type EditBlog = DocumentNode;
```

> Edit Blog Node

```ts
type EditBlogNode = DocumentNode;
```

> Delete blog node

```ts
type DeleteBlogNode = {
  delete_node: {
    uid: number;
  };
  update_node: {
    uid: number;
    parent_id: number;
  };
};
```

# **Book**

> Create Book

```ts
type CreateBook = DocumentNode;
```

> Add Book Node

```ts
type AddBookNode = {
  new_node: DocumentNode;
  update_node: {
    uid: number;
    parent_id: null | number;
  };
};
```

> Edit Book

```ts
type EditBook = DocumentNode;
```

> Edit Book Node

```ts
type EditBookNode = DocumentNode;
```

> Delete Book node

```ts
type DeleteBookNode = {
  delete_nodes: number[];
  update_node: {
    uid: number;
    parent_id: number;
  };
};
```
