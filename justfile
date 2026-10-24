set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]
set dotenv-load := false

# Lista las recetas disponibles
default:
    @just --list

# Instala las dependencias de todos los stacks
install:
    pnpm install
    uv --directory services/api sync

# Levanta PostgreSQL local
up:
    docker compose -f infra/compose/docker-compose.yml up -d

# Apaga los contenedores locales
down:
    docker compose -f infra/compose/docker-compose.yml down

# Arranca las tres apps web en paralelo (4321, 4322, 4323)
dev:
    pnpm dev

# Arranca la API con recarga en el puerto 8000
api:
    uv --directory services/api run uvicorn tchat_api.main:app --reload --port 8000

# Arranca el cliente de terminal (requiere `just cli-login` antes)
cli:
    cargo run --manifest-path apps/cli/Cargo.toml -p tchat-cli --

# Autoriza este terminal contra la API local
cli-login:
    cargo run --manifest-path apps/cli/Cargo.toml -p tchat-cli -- login

# Aplica las migraciones pendientes
migrate:
    uv --directory services/api run alembic upgrade head

# Crea una migración autogenerada: just revision "mensaje"
revision msg:
    uv --directory services/api run alembic revision --autogenerate -m "{{msg}}"

# Ejecuta los tests de la API (requiere Postgres: just up)
test:
    uv --directory services/api run pytest

# Exporta OpenAPI y regenera el cliente TS
gen-client:
    uv --directory services/api run python -m tchat_api.export_openapi
    pnpm --filter @tchat/api-client gen

# Construye la imagen de la API en local
image:
    docker build -t tchat-api:local services/api

# Comprueba formato, clippy y tests del cliente Rust
check-cli:
    cargo fmt --manifest-path apps/cli/Cargo.toml --all --check
    cargo clippy --manifest-path apps/cli/Cargo.toml --workspace --all-targets -- -D warnings
    cargo test --manifest-path apps/cli/Cargo.toml --workspace

# Verifica formato, lint y tipos de todo el monorepo
check:
    uv --directory services/api run ruff format --check .
    uv --directory services/api run ruff check .
    uv --directory services/api run mypy src
    pnpm lint
    pnpm format:check
    pnpm -r --if-present typecheck
    cargo fmt --manifest-path apps/cli/Cargo.toml --all --check
    cargo clippy --manifest-path apps/cli/Cargo.toml --workspace --all-targets -- -D warnings
    cargo test --manifest-path apps/cli/Cargo.toml --workspace

# Compila todas las apps web
build:
    pnpm -r --if-present build
