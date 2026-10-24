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

# Arranca la API con recarga en el puerto 8000
api:
    uv --directory services/api run uvicorn tchat_api.main:app --reload --port 8000

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

# Verifica formato, lint y tipos de todo el monorepo
check:
    uv --directory services/api run ruff format --check .
    uv --directory services/api run ruff check .
    uv --directory services/api run mypy src
    pnpm -r --if-present lint
    pnpm -r --if-present typecheck

# Compila todas las apps web
build:
    pnpm -r --if-present build
