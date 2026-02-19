# Suggested Commands for Claude Agent SDK Rust

## Development Commands

### Build
```bash
cargo build --workspace           # Debug build
cargo build --workspace --release # Release build (for benchmarks)
```

### Test
```bash
cargo test --workspace            # Run all tests
cargo test --workspace --release  # Release mode tests
cargo test test_name --workspace  # Run specific test
```

### Lint & Format
```bash
cargo fmt --workspace             # Format code
cargo fmt --workspace -- --check  # Check formatting
cargo clippy --workspace --all-targets  # Lint
cargo clippy --workspace --all-targets --fix  # Auto-fix clippy warnings
```

### Documentation
```bash
cargo doc --workspace --open      # Build and open docs
```

### Examples (via Justfile)
```bash
just build-examples               # Build all examples
just run <example_name>           # Run specific example
just verify                       # Full verification (build + test)
```

### Performance Testing
```bash
just bench-build                  # Build release for benchmarking
just bench-quick                  # Quick performance test
just bench-detailed               # Detailed benchmark (5 iterations)
just bench-all                    # Complete performance suite
```

## Git Commands (Darwin/macOS)
```bash
git status
git branch
git diff
git add -p                        # Interactive staging
git commit -m "message"
git push origin branch_name
```

## System Commands
```bash
ls -la                            # List files
find . -name "*.rs"               # Find Rust files
grep -r "pattern" crates/         # Search pattern
```

## Pre-commit Checklist
1. `cargo fmt --workspace`
2. `cargo clippy --workspace --all-targets`
3. `cargo test --workspace`
4. Verify no API key in code: `git grep "sk-ant-"`