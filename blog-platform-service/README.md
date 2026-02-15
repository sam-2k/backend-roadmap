# Blogging Platform API

A high-performance RESTful API built with **Rust** and the **Rocket** web framework for [roadmap.sh's Blogging Platform API project](https://roadmap.sh/projects/blogging-platform-api). This project focuses on type-safe endpoint handling, SQLite persistence, and structured data management.

## 🚀 Features

- **Full CRUD Support**: Complete implementation for Creating, Reading, Updating, and Deleting blog posts
- **Soft Delete Logic**: Built-in database logic to flag posts as deleted without physically removing them
- **Search Functionality**: Search blog posts by title using query parameters
- **Data Validation**: Input validation using the `validator` crate
- **Persistent Storage**: Utilizes **SQLite** for reliable, serverless data management
- **Automated Testing**: Includes comprehensive test scenarios for all operations
- **Type Safety**: Leverages Rust's type system for compile-time guarantees

## 🛠️ Tech Stack

- **Backend**: [Rust](https://www.rust-lang.org/)
- **Web Framework**: [Rocket.rs](https://rocket.rs/) v0.5.1
- **Database**: SQLite via [rusqlite](https://github.com/rusqlite/rusqlite)
- **Validation**: [validator](https://github.com/Keats/validator) v0.16
- **Time Handling**: [time](https://github.com/time-rs/time) v0.3

## 📋 API Endpoints

All endpoints are prefixed with `/posts`.

| Method   | Endpoint               | Description                                      |
| :------- | :--------------------- | :----------------------------------------------- |
| `GET`    | `/posts`               | List all active blog posts (returns titles only) |
| `GET`    | `/posts?term=<search>` | Search blog posts by title                       |
| `GET`    | `/posts/<id>`          | Retrieve a specific post by its ID               |
| `POST`   | `/posts`               | Create a new blog post                           |
| `PUT`    | `/posts/<id>`          | Update an existing post                          |
| `DELETE` | `/posts/<id>`          | Soft-delete a post (sets `deleted_at` timestamp) |

### Request/Response Examples

#### Create a Blog Post

**Request:**

```bash
POST /posts
Content-Type: application/json

{
  "title": "My First Blog Post",
  "content": "This is the content of my first blog post.",
  "category": "Technology",
  "tags": ["rust", "api", "web"]
}
```

**Response:**

```json
{
  "id": 1,
  "title": "My First Blog Post",
  "content": "This is the content of my first blog post.",
  "category": "Technology",
  "tags": ["rust", "api", "web"],
  "createdAt": "2024-01-15T10:30:00Z",
  "updatedAt": "2024-01-15T10:30:00Z"
}
```

#### Get a Blog Post

**Request:**

```bash
GET /posts/1
```

**Response:**

```json
{
  "id": 1,
  "title": "My First Blog Post",
  "content": "This is the content of my first blog post.",
  "category": "Technology"
}
```

#### Update a Blog Post

**Request:**

```bash
PUT /posts/1
Content-Type: application/json

{
  "title": "My Updated Blog Post",
  "content": "This is the updated content.",
  "category": "Programming",
  "tags": ["rust", "api"]
}
```

**Response:**

```json
{
  "title": "My Updated Blog Post",
  "content": "This is the updated content.",
  "category": "Programming"
}
```

#### Delete a Blog Post

**Request:**

```bash
DELETE /posts/1
```

**Response:**

```json
{
  "success": true
}
```

#### Search Blog Posts

**Request:**

```bash
GET /posts?term=rust
```

**Response:**

```json
{
  "title": "My First Blog Post",
  "content": "This is the content of my first blog post.",
  "category": "Technology"
}
```

## ⚙️ Getting Started

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) (version 1.70 or higher recommended)
- SQLite3 (optional, for manual database inspection)

### Installation

1. Clone the repository:

   ```bash
   git clone <your-repo-url>
   cd blog-platform-service
   ```

2. Build the project:
   ```bash
   cargo build
   ```

### Running the Application

Start the server:

```bash
cargo run
```

The API will be available at `http://localhost:8000`.

### Running in Development Mode

For development with auto-reload:

```bash
cargo watch -x run
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

The test suite includes:

- CRUD operation tests
- Soft delete functionality tests
- Search functionality tests
- Data validation tests

## 📊 Database Schema

The application uses an in-memory SQLite database with the following schema:

```sql
CREATE TABLE blogs (
    id          INTEGER PRIMARY KEY,
    title       TEXT NOT NULL,
    content     TEXT NOT NULL,
    category    TEXT NOT NULL,
    deleted_at  TEXT DEFAULT NULL
);
```

**Note:** The current implementation uses an in-memory database. To persist data across restarts, modify `db.rs` to use `Connection::open("blog.db")` instead of `Connection::open_in_memory()`.

### Inspecting the Database

If you switch to a file-based database, you can inspect it using SQLite CLI:

```bash
sqlite3 blog.db

# View all posts
SELECT * FROM blogs;

# View only active posts
SELECT * FROM blogs WHERE deleted_at IS NULL;

# View deleted posts
SELECT * FROM blogs WHERE deleted_at IS NOT NULL;
```

## 📁 Project Structure

```
blog-platform-service/
├── src/
│   ├── main.rs       # Application entry point and route handlers
│   ├── models.rs     # Data models and structures
│   ├── db.rs         # Database connection and setup
│   ├── validate.rs   # Custom validation logic
│   └── tests.rs      # Test suite
├── Cargo.toml        # Project dependencies and metadata
└── README.md         # Project documentation
```

## 🔍 Data Model

### Blog Post Structure

```rust
{
    id: u32,                    // Auto-generated ID
    title: String,              // Required, min length: 1
    content: String,            // Required, min length: 1
    category: String,           // Required, min length: 1
    tags: Vec<String>,          // Required, each tag min length: 1
    created_at: String,         // Auto-generated (RFC3339 format)
    updated_at: String,         // Auto-updated (RFC3339 format)
}
```

### Validation Rules

- `title`: Required, minimum length of 1 character
- `content`: Required, minimum length of 1 character
- `category`: Required, minimum length of 1 character
- `tags`: Required array with at least one tag, each with minimum length of 1

## 🛡️ Error Handling

The API returns appropriate HTTP status codes:

- `200 OK`: Successful operation
- `400 Bad Request`: Invalid input or blog post not found
- `500 Internal Server Error`: Database or server errors

Error responses follow this format:

```json
"Error message describing what went wrong"
```

## 🔧 Configuration

### Changing to Persistent Database

To use a file-based database instead of in-memory:

1. Open `src/db.rs`
2. Change line 48 from:
   ```rust
   let conn: Option<Connection> = Connection::open_in_memory().ok();
   ```
   to:
   ```rust
   let conn: Option<Connection> = Connection::open("blog.db").ok();
   ```

### Changing Port

To change the default port (8000), set the `ROCKET_PORT` environment variable:

```bash
ROCKET_PORT=3000 cargo run
```

## 📝 Development Notes

- The application uses atomic counters for generating blog IDs
- Soft deletes are implemented by setting the `deleted_at` timestamp
- All database operations use prepared statements to prevent SQL injection
- The project follows Rust's ownership and borrowing principles for memory safety

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is created as part of the [roadmap.sh](https://roadmap.sh) backend project challenges.

## 🎯 Roadmap.sh Project Requirements

This implementation fulfills the following requirements from the [Blogging Platform API](https://roadmap.sh/projects/blogging-platform-api) project:

- ✅ Create a new blog post
- ✅ Update an existing blog post
- ✅ Delete an existing blog post
- ✅ Get a single blog post
- ✅ Get all blog posts
- ✅ Search blog posts by term

## 🚀 Future Enhancements

Potential improvements for this project:

- [ ] Add pagination for listing posts
- [ ] Implement proper tag management (many-to-many relationship)
- [ ] Add authentication and authorization
- [ ] Include created/updated timestamps in all responses
- [ ] Add filtering by category
- [ ] Implement full-text search
- [ ] Add rate limiting
- [ ] Create OpenAPI/Swagger documentation
- [ ] Add database migrations system
- [ ] Deploy to production environment

## 📚 Resources

- [Rocket.rs Documentation](https://rocket.rs/v0.5/guide/)
- [Rusqlite Documentation](https://docs.rs/rusqlite/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Roadmap.sh Backend Projects](https://roadmap.sh/backend/projects)
