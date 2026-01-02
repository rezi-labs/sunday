# Agent Guidelines for Taste Codebase

## Files to Exclude

When working with this codebase, agents should exclude these large generated/vendor files:

- `assets/tw.js`
- `assets/daisy.css`
- `assets/htmx.js`

**Important**: Always exclude these files when scanning or searching the project to avoid processing large generated files that aren't relevant for code analysis or modifications.

## Build Commands

- `just run` - Run development server
- `just watch` - Run with auto-reload (preferred for development)
- `just test` - Run all tests (currently no tests exist)
- `just lint` - Check formatting and run clippy
- `just fmt` - Format code and fix issues
- `just verify` - Run lint + test (matches CI)

## Code Style & Conventions

- Organize code: `db/` (entities), `routes/` (handlers), `view/` (templates)
- Import order: std, external crates, internal modules
- Use `Result<HttpResponse>` for route handlers
- Database operations use SeaORM with proper error handling
- Validate forms with `validator` crate, return JSON errors
- Use `log::info/warn/error` for logging, not `println!`
- Session management via actix-session with database storage
- Use `async move` blocks for async handlers
- Database entities: derive Serialize/Deserialize, use Uuid primary keys