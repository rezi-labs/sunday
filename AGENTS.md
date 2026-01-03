# Agent Guidelines Codebase

## Documentation

For LLM stuff always use rig-core and rig-postgres
https://github.com/0xPlaygrounds/rig/tree/main/rig-integrations/rig-postgres


## Build Commands

- `just run` - Run development server
- `just watch` - Run with auto-reload (preferred for development)
- `just test` - Run all tests (currently no tests exist)
- `just lint` - Check formatting and run clippy
- `just fmt` - Format code and fix issues
- `just verify` - Run lint + test (matches CI)

## Code Style & Conventions

- Import order: std, external crates, internal modules
- Use `Result<HttpResponse>` for route handlers
- Use `log::info/warn/error` for logging, not `println!`
- Use `async move` blocks for async handlers
- Database entities: derive Serialize/Deserialize
