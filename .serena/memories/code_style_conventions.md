# Code Style and Conventions - Claude Agent SDK Rust

## Naming Conventions
- **Snake_case** for functions, variables, modules
- **PascalCase** (CamelCase) for types, traits, structs, enums
- **SCREAMING_SNAKE_CASE** for constants
- Use descriptive names, avoid abbreviations

## Rust Idioms
- Use `Result<T, E>` for error handling
- Use `Option<T>` for nullable values
- Prefer `?` operator over `match` for error propagation
- Use `async/await` with tokio runtime
- Use `Arc<Mutex<T>>` for shared mutable state

## Documentation
- Use `//!` for module-level docs
- Use `///` for item-level docs
- Include code examples in doc comments
- Document public APIs with examples

## Error Handling Pattern
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClaudeError {
    #[error("CLI not found: {0}")]
    CliNotFound(#[from] CliNotFoundError),
    #[error("Connection failed: {0}")]
    Connection(#[from] ConnectionError),
}
```

## Builder Pattern
Use `typed-builder` crate for builder pattern:
```rust
#[derive(typed_builder::TypedBuilder)]
pub struct ClaudeAgentOptions {
    #[builder(default, setter(into))]
    pub model: Option<String>,
}
```

## Async Patterns
- Use `tokio::sync::Mutex` for async contexts
- Use `async_trait` for async trait methods
- Prefer `tokio::spawn` for concurrent tasks

## Testing
- Unit tests in same file with `#[cfg(test)]`
- Integration tests in `tests/` directory
- Use `#[tokio::test]` for async tests

## Module Organization
```
mod.rs          # Re-exports and module docs
types.rs        # Type definitions
error.rs        # Error types
impl_xxx.rs     # Implementation details
```