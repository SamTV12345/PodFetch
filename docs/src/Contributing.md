# Contributing

## Preamble

First of all, thank you for considering contributing to Podfetch. It is people like you that make Podfetch great.
I appreciate every contribution, no matter how small it is. If you have any questions, don't hesitate to ask them in 
the discussions section.

## Building the project

### Prerequisites
- Rust
- Cargo
- Node
- npm/yarn/pnpm

### Building the app
```bash
# File just needs to be there
touch static/index.html
cargo.exe run --color=always --package podfetch --bin podfetch
cd ui
<npm/yarn/pnpm> install
<npm/yarn/pnpm> run dev
```