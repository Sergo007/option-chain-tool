# option-chain-tool

A Rust macro that brings JavaScript-like optional chaining to Rust, allowing you to use the `?` operator in any context - even in functions that don't return `Option` or `Result`.

## Motivation

In Rust, the `?` operator is a powerful tool for ergonomic error handling, but it comes with a significant limitation: **it only works inside functions that return `Option` or `Result`**. This restriction forces developers into verbose workarounds when working with nested optional values in regular functions.

### The Problem

Consider a common scenario: you have deeply nested optional structures and want to safely access a value deep within. In Rust, you typically have three options, all with drawbacks:

1. **Verbose chaining with `and_then` and `map`**:
```rust
let value = test_struct.value
    .and_then(|v1| v1.value)
    .and_then(|v2| v2.value);
```
This becomes increasingly unreadable as nesting increases.

2. **Manual unwrapping with pattern matching**:
```rust
if let Some(____v) = &test_struct.value {
    if let Some(____v) = &____v.value {
        Some(&____v.value)
    } else {
        None
    }
} else {
    None
}
```
Even more verbose and error-prone.

3. **Wrapping everything in a helper function**:
```rust
fn get_nested_value(test_struct: &TestStruct) -> Option<i32> {
    test_struct.value?.value?.value
}
let value = get_nested_value(&test_struct);
```
Creates unnecessary function overhead for simple access patterns.

### The Solution

JavaScript and TypeScript developers enjoy optional chaining (`?.`) that makes this trivial:
```javascript
const value = test_struct?.value?.value?.value;
```

**option-chain** brings this ergonomic experience to Rust with a simple macro that lets you use the natural `?` operator anywhere:

```rust
let value = opt!(test_struct.value?.value?.value?);
```

Clean, readable, and efficient - without requiring your function to return `Option`.

## Features

- 🪶 **Lightweight**: Just a use native rust TokenStream, zero dependencies.
- 🚀 **Zero overhead**: Compiles down to the same code as manual `if let Some(...) = ...` chains.
- 🎯 **Intuitive**: Uses Rust's familiar `?` operator syntax.
- 🔒 **Type-safe**: Full compile-time type checking.
- 📦 **Works everywhere**: Use in functions returning `()`, concrete types, or anything else.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
option-chain-tool = "0.10"
```

## Usage

### Basic Example

```rust
use option_chain_macro::opt;

#[derive(Debug, Clone)]
struct User {
    profile: Option<Profile>,
}

#[derive(Debug, Clone)]
struct Profile {
    address: Option<Address>,
}

#[derive(Debug, Clone)]
struct Address {
    city: Option<String>,
}

fn main() {
    let user = User {
        profile: Some(Profile {
            address: Some(Address {
                city: Some("New York".to_string()),
            }),
        }),
    };

    // ✅ With opt! macro - clean and readable
    let city = opt!(user.profile?.address?.city?);
    println!("City: {:?}", city); // City: Some("New York")

    // ❌ Without macro - verbose and nested
    let city_verbose = user.profile
        .and_then(|p| p.address)
        .and_then(|a| a.city);
}
```

### Advanced Examples

#### Working with Vectors and Methods

```rust
use option_chain_macro::opt;

#[derive(Debug, Clone)]
struct Team {
    members: Option<Vec<Member>>,
}

#[derive(Debug, Clone)]
struct Member {
    name: String,
    email: Option<String>,
}

fn main() {
    let team = Team {
        members: Some(vec![
            Member { 
                name: "Alice".to_string(), 
                email: Some("alice@example.com".to_string()) 
            },
            Member { 
                name: "Bob".to_string(), 
                email: None 
            },
        ]),
    };

    // Access nested vector elements safely
    let first_email = opt!(team.members?.get(0)?.email?);
    println!("First email: {:?}", first_email); // Some("alice@example.com")

    let second_email = opt!(team.members?.get(1)?.email?);
    println!("Second email: {:?}", second_email); // None
}
```

#### Accessing Required Fields Through Optional Chains

```rust
use option_chain_macro::opt;

#[derive(Debug, Clone)]
struct Config {
    database: Option<DatabaseConfig>,
}

#[derive(Debug, Clone)]
struct DatabaseConfig {
    host: String,          // Required field
    port: i32,            // Required field
    credentials: Option<Credentials>,
}

#[derive(Debug, Clone)]
struct Credentials {
    username: String,
}

fn main() {
    let config = Config {
        database: Some(DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            credentials: None,
        }),
    };

    // Access required fields through optional chain
    let port = opt!(&config.database?.port);
    println!("Port: {:?}", port); // Some(5432)

    let host = opt!(&config.database?.host);
    println!("Host: {:?}", host); // Some("localhost")
}
```

### Real-World Scenario

```rust
use option_chain_macro::opt;

#[derive(Debug, Clone)]
struct ApiResponse {
    data: Option<ResponseData>,
}

#[derive(Debug, Clone)]
struct ResponseData {
    user: Option<UserData>,
}

#[derive(Debug, Clone)]
struct UserData {
    id: i32,
    metadata: Option<Metadata>,
}

#[derive(Debug, Clone)]
struct Metadata {
    last_login: Option<String>,
}

// Regular function that doesn't return Option
fn process_response(response: ApiResponse) {
    // Without opt! macro, you'd need verbose chaining or helper functions
    // With opt! macro, it's clean and expressive
    
    let user_id = opt!(&response.data?.user?.id);
    let last_login = opt!(response.data?.user?.metadata?.last_login?);
    
    println!("User ID: {:?}", user_id);           // Some(42)
    println!("Last login: {:?}", last_login);    // Some("2024-01-01")
}
```

## Comparison

**JavaScript/TypeScript:**
```javascript
const city = user?.profile?.address?.city;
```

**Rust without option-chain:**
```rust
let city = user.profile
    .and_then(|p| p.address)
    .and_then(|a| a.city);
```

**Rust with option-chain:**
```rust
let city = opt!(user.profile?.address?.city?);
```

## How It Works

The `opt!` macro transforms your code at compile time, converting the intuitive `?` syntax into efficient `if let Some(...) = ...` chains. There's no runtime overhead - it's purely a syntactic convenience.

```rust
// What you write:
let a: Option<&String> = opt!(user.profile?.address?.city?);

// What the compiler sees (roughly):
let a: Option<&String> = if let Some(____v) = &user.profile {
    if let Some(____v) = &____v.address {
        if let Some(____v) = &____v.city {
            Some(____v)
        } else {
            None
        }
    } else {
        None
    }
} else {
    None
};
```

## When to Use

**✅ Use option-chain when:**
- Working with nested optional structures in functions that don't return `Option`
- You want to avoid verbose `and_then` chains
- Accessing data from deeply nested API responses or configs
- You need readable optional access in utility functions

**❌ Consider alternatives when:**
- Your function already returns `Option` or `Result` (use native `?` operator)
- You're working with flat structures (no nesting)
- You need custom error messages (consider explicit error handling)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## See Also

- [Rust `Option` documentation](https://doc.rust-lang.org/std/option/enum.Option.html)
- [The `?` operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator)
- [Optional chaining in JavaScript](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Optional_chaining)
