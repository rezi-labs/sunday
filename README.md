# sunday
RAG Chat

## Development

### Prerequisites
- Docker & Docker Compose (for PostgreSQL with pgvector)
- Rust toolchain

### Quick Start

1. **Set up environment variables** (copy and configure):
    > set FAKE_AI for simple testing

   ```bash
   cp .env.example .env
   ```

2. **Start development** (this will automatically start the database):
   ```bash
   just run
   ```
   Or with auto-reload:
   ```bash
   just watch
   ```

3. **Set up database** (first time only):
   ```bash
   just db-setup
   ```

### Database Commands

- `just db-setup` - Create database and run migrations
- `just db-migrate` - Run migrations
- `just db-reset` - Drop and recreate database
- `just db-stop` - Stop the PostgreSQL container
- `just db-logs` - View database logs

### Other Commands

- `just lint` - Check formatting and run clippy
- `just fmt` - Format code and fix issues
- `just test` - Run tests
- `just verify` - Run lint + test (matches CI)

