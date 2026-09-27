---
order: 5.3
name: CI cross-build
---
- **Slug:** ci-crossbuild
- **Goal:** GitHub Actions em windows-latest e ubuntu-latest a cada PR/push: build, `cargo test`, clippy `-D warnings`, fmt check, cache cargo, cobertura via cargo-llvm-cov com gate 80%, build do app de bandeja nos dois SOs; workflow reutilizável (template) no repositório `slipalison/github-workflows`, chamado pelo ddc-control
