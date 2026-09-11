---
order: 4
name: CI cross-build
---
- **Slug:** ci-crossbuild
- **Goal:** GitHub Actions em windows-latest e ubuntu-latest: build, `cargo test`, clippy `-D warnings`, fmt check, cache cargo, cobertura via cargo-llvm-cov com gate 80%
