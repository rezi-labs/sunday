# sunday
RAG Chat API with pgvector

## Development

### Prerequisites
- Docker & Docker Compose (for PostgreSQL with pgvector)
- Rust toolchain

### Quick Start

1. **Set up environment variables**:
   ```bash
   cp .env.example .env
   ```
   
   Configure your `.env` file:
   - Set `FAKE_AI=true` for simple testing without Azure OpenAI
   - Or configure Azure OpenAI credentials for full AI features

2. **Start development**:
   ```bash
   just run
   ```
   This will:
   - Stop any running database containers
   - Start a fresh PostgreSQL container with pgvector
   - Run database migrations automatically
   - Start the server

   For development with auto-reload:
   ```bash
   just watch
   ```

### Available Commands

#### Development
- `just run` - Run development server
- `just watch` - Run with auto-reload (preferred for development)
- `just ui` - Open test UI in browser

#### Code Quality
- `just lint` - Check formatting and run clippy
- `just fmt` - Format code and fix issues
- `just test` - Run all tests
- `just verify` - Run lint + test (matches CI)

#### Database
- `just db-start` - Start PostgreSQL container
- `just db-stop` - Stop PostgreSQL container
- `just db-logs` - View database logs

Note: Migrations run automatically when the server starts.

