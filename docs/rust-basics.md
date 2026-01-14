# Rust Basics for This Project

New to Rust? No worries. This covers the key stuff you'll run into while working on Cohort Tracker.

## What's Rust anyway?

Rust is a programming language that's really picky about safety and performance. It catches a lot of bugs at compile time that would crash your program later. Think of it as having a very thorough code reviewer built into the compiler.

## The important concepts

### 1. Ownership (or "who owns this data?")

```rust
// Each piece of data has exactly one owner
let name = String::from("Alice");
let name2 = name; // name is "moved" - you can't use it anymore

// But you can "borrow" data temporarily
fn print_name(name: &String) {
    println!("{}", name);
}
```

**What you'll see:** Lots of `&str` and `&String` in our function parameters. These are borrowed references - we're just looking at the data, not taking ownership.

### 2. Results (handling things that might fail)

```rust
// Instead of exceptions, Rust uses Result types
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Can't divide by zero!".to_string())
    } else {
        Ok(a / b)
    }
}

// Handle it like this
match divide(10, 2) {
    Ok(result) => println!("Got: {}", result),
    Err(error) => println!("Oops: {}", error),
}
```

**In our code:** Every database call and API request returns a `Result`. You'll see this everywhere.

### 3. Options (for things that might not exist)

```rust
// Option<T> is Rust's way of saying "this might be null"
let maybe_number: Option<i32> = Some(42);
let no_number: Option<i32> = None;

// Check what you got
if let Some(num) = maybe_number {
    println!("Found a number: {}", num);
}
```

**In our code:** Database fields like `reviewed_at` use `Option<String>` since students might not have reviewed assignments yet.

### 4. Structs (custom data types)

```rust
// Bundle related data together
struct Student {
    id: String,
    name: String,
    email: String,
}

// Add methods to your structs
impl Student {
    fn new(id: String, name: String, email: String) -> Self {
        Student { id, name, email }
    }
    
    fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.email)
    }
}
```

**In our code:** We have structs for `Student`, `Assignment`, `Progression`, and more.

### 5. Traits (shared behavior)

```rust
// Define what types can do
trait Display {
    fn display(&self) -> String;
}

impl Display for Student {
    fn display(&self) -> String {
        self.display_name()
    }
}
```

**In our code:** We use traits like `Serialize` and `Deserialize` to convert data to/from JSON.

### 6. Async/await (non-blocking operations)

```rust
// For operations that take time (like network requests)
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

// Run async code
#[tokio::main]
async fn main() {
    match fetch_data("https://api.example.com").await {
        Ok(data) => println!("Got: {}", data),
        Err(e) => println!("Failed: {}", e),
    }
}
```

**In our code:** All the OpenClass API calls are async.

## Patterns you'll see everywhere

### The `?` operator (error propagation)

```rust
// Real example from our config.rs
pub fn from_file(path: &str) -> Result<Self> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
    toml::from_str(&content).map_err(|e| anyhow!("Failed to parse config: {}", e))
}
```

Much cleaner than checking every error manually. The `?` says "if this fails, return early with the error."

### Pattern matching

```rust
// From our main.rs
match cli.command {
    cli::Commands::Init { email, password, api_base } => {
        cli::handle_init(email, password, api_base).await?;
    }
    cli::Commands::List { all } => {
        cli::handle_list(all).await?;
    }
    cli::Commands::Sync { class } => {
        cli::handle_sync(cli.config, class).await?;
    }
    cli::Commands::Status => {
        cli::handle_status(cli.config).await?;
    }
}
```

### Working with Options

```rust
// Real code for finding home directory
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .and_then(|h| if h.is_empty() { None } else { Some(h) })
        .map(PathBuf::from)
}
```

This chains operations on `Option` - if any step fails, the whole thing returns `None`.

This reads like: "take progressions, keep only the passing ones, grab their student IDs, and collect into a vector."

## Rust Tools You'll Use

### Cargo (Package Manager)

```bash
cargo build          # Compile the project
cargo run            # Build and run
cargo test           # Run tests
cargo check          # Check for errors without building
```

### Common Commands in Our Project

```bash
cargo run -- init --email user@example.com --password pass --class-id 123
cargo run -- sync
cargo run -- status
```

## Learning Resources

- [The Rust Book](https://doc.rust-lang.org/book/) - Official tutorial
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Code examples
- [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises

## Next Steps

Once you understand these basics, check out:
- [Project Architecture](./architecture.md) - How our code is organized
- [Why Rust?](./why-rust.md) - Why we chose Rust for this project
