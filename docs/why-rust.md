# Why Rust? Our Technology Decisions

So why did we pick Rust for this project? And what does that actually mean for you as a developer? Let me break it down.

## The short answer

We wanted something that wouldn't crash on us during long sync operations, and we wanted to learn something new. Rust delivers on both fronts, but it comes with a learning curve.

## What's great about Rust

### 1. **It catches bugs before they bite you**

```rust
// This won't even compile - Rust stops you from shooting yourself in the foot
let data = vec![1, 2, 3];
let reference = &data[0];
drop(data);  // Error: can't drop data while it's borrowed
println!("{}", reference);
```

No more mysterious crashes from null pointers or memory corruption. The compiler is like having a really pedantic code reviewer who never gets tired.

### 2. **Errors are handled explicitly**

```rust
// You can't ignore errors - the compiler makes you deal with them
fn sync_data() -> Result<(), SyncError> {
    let token = authenticate().await?;  // Handle auth errors
    let data = fetch_data(&token).await?;  // Handle network errors
    save_to_db(&data)?;  // Handle database errors
    Ok(())
}
```

No more silent failures that you only discover in production.

### 3. **Fast code that looks high-level**

```rust
// This reads nicely but compiles to very efficient machine code
let passing_students: Vec<_> = progressions
    .iter()
    .filter(|p| p.grade.unwrap_or(0.0) >= 0.7)
    .collect();
```

You get the expressiveness of Python with the speed of C.

### 4. **Async that actually works**

```rust
// Handle thousands of concurrent requests without callback hell
let futures = urls.iter().map(|url| fetch_data(url));
let results = futures::future::join_all(futures).await;
```

### 5. **The type system catches logic errors**

```rust
// The compiler ensures you handle all cases
struct Progression {
    student_id: String,      // Always present
    grade: Option<f64>,      // Might be null - compiler forces you to check
    completed_at: DateTime,  // Type-safe dates
}
```

### 6. **Great tooling out of the box**

```bash
cargo build    # Just works
cargo test     # Runs all tests
cargo fmt      # Formats your code
cargo clippy   # Catches common mistakes
```

No configuration hell, no hunting for the right linter setup.

## The not-so-great parts

### 1. **The learning curve is real**

```rust
// This is confusing when you're starting out
fn process_data(data: Vec<String>) -> Vec<String> {
    // data gets "moved" here - you can't use it after this function
    data.into_iter().map(|s| s.to_uppercase()).collect()
}

// Better approach: borrow instead of taking ownership
fn process_data(data: &[String]) -> Vec<String> {
    data.iter().map(|s| s.to_uppercase()).collect()
}
```

It takes time to learn when to use `&`, `&mut`, or owned values. I still get this wrong sometimes.

### 2. **Lifetime annotations can be confusing**

```rust
// Sometimes you need to tell Rust how long references live
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

Fortunately, you don't need this often in our project.

### 3. **Async can get weird**

```rust
// The actual type is much more complex than it looks
async fn fetch_data() -> Result<String, reqwest::Error> { /* ... */ }

// Real type: impl Future<Output = Result<String, reqwest::Error>>
```

Error messages can be cryptic when async types don't line up.

### 4. **Compilation takes forever**

```bash
$ cargo build
   Compiling cohort-tracker v0.1.0
   # Go grab coffee... still compiling...
```

Especially on the first build. It gets better with incremental compilation, but it's still slower than Python or JavaScript.

## What we could have used instead

### Python + Requests + SQLite

```python
# Much simpler to write
import requests
import sqlite3

def sync_data():
    response = requests.get(f"{API_BASE}/progressions")
    data = response.json()
    
    conn = sqlite3.connect("data.db")
    for item in data:
        conn.execute("INSERT INTO progressions ...", item)
```

**Good stuff:**
- Way faster to write initially
- Huge ecosystem of libraries
- Easy to learn and modify
- Great for prototyping

**The downsides:**
- Runtime errors (typos crash your program)
- Slower for large datasets
- Dependency management can be a nightmare
- No compile-time safety net

### Node.js + TypeScript

```typescript
// Familiar syntax with some type safety
interface Progression {
    studentId: string;
    grade?: number;
    completedAt: Date;
}

async function syncData(): Promise<void> {
    const response = await fetch(`${API_BASE}/progressions`);
    const data: Progression[] = await response.json();
    // ... database stuff
}
```

**Good stuff:**
- Familiar to web developers
- Good async support
- Large ecosystem
- TypeScript adds some type safety

**The downsides:**
- Still need runtime type checking
- Memory usage gets heavy with large datasets
- npm security is... interesting
- null/undefined errors still happen

### Go

```go
// Simple and fast
func syncData() error {
    resp, err := http.Get(apiURL)
    if err != nil {
        return err
    }
    defer resp.Body.Close()
    
    var data []Progression
    return json.NewDecoder(resp.Body).Decode(&data)
}
```

**Good stuff:**
- Simple syntax
- Fast compilation
- Great for CLI tools
- Excellent concurrency

**The downsides:**
- Verbose error handling (lots of `if err != nil`)
- Limited type system
- Less expressive than Rust

## So why did Rust win?

### 1. **We need reliability**

This tool runs unattended syncs. When it crashes at 2 AM, someone has to wake up and fix it. Rust's compile-time guarantees mean way fewer middle-of-the-night emergencies.

### 2. **Performance actually matters**

Syncing 4,000+ records with rate limiting takes time. Rust's efficiency means faster syncs and less resource usage. That adds up over time.

### 3. **Learning opportunity**

This is an educational project. Rust teaches you valuable concepts about memory management and systems programming that you won't get from higher-level languages.

### 4. **Future-proofing**

As we add the REST API (Phase 2) and analytics (Phase 3), Rust's performance and safety become even more valuable.

### 5. **Single binary deployment**

```bash
# No runtime dependencies, no version conflicts
cargo build --release
./target/release/cohort-tracker sync
```

Just copy one file and you're done. No Python virtual environments, no Node.js version managers.

## The trade-offs we made

**Development speed vs. runtime safety:** We chose slower initial development for compile-time guarantees. Better to catch bugs during development than in production.

**Learning curve vs. long-term maintainability:** We picked the steeper learning curve for better code quality. Since this is educational, learning Rust is part of the value.

**Ecosystem maturity vs. performance:** Rust's ecosystem is smaller, but our dependencies (HTTP client, SQLite, JSON) are solid.

## When Rust might be the wrong choice

**Rapid prototyping:** If you need to validate an idea quickly, Python or JavaScript are probably better.

**Team expertise:** If your team is all web developers, TypeScript might have lower onboarding costs.

**Simple scripts:** For one-off data processing, Python's simplicity often wins.

**Heavy string processing:** Languages with garbage collectors can be more ergonomic for text-heavy work.

## What we learned

### What worked well

- **Error handling:** `Result` types caught tons of bugs early
- **Async:** Tokio made concurrent API calls straightforward once we figured it out
- **Tooling:** Cargo made dependency management painless
- **Performance:** Sync times are great even with rate limiting

### What was painful

- **Initial setup:** Getting async + database + HTTP client working together took forever
- **Error types:** Defining custom error enums for different failure modes was tedious
- **Borrowing:** Learning when to use `&str` vs `String` was confusing
- **Debugging:** Async stack traces can be cryptic

### What we'd do differently next time

- **Start simpler:** Begin with synchronous code, add async later
- **More examples:** Include more code examples in documentation from day one
- **Better error messages:** Invest in user-friendly error messages earlier
- **Testing:** Set up integration tests from the beginning

## Bottom line

Rust was the right choice for Cohort Tracker because:

1. **Reliability** matters more than development speed for this use case
2. **Performance** is important when dealing with large datasets  
3. **Learning** Rust provides educational value beyond just this project
4. **Safety** prevents production issues that would be hard to debug

The learning curve is real, but the benefits justify the investment for a production tool that needs to run reliably over time.

## What's next?

- Read [Development Guide](./development.md) if you want to contribute
- Check [Database Design](./database.md) for schema details
- Review the main [README.md](../README.md) for setup instructions
