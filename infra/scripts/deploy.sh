#!/usr/bin/env bash
set -euo pipefail

STACK_DIR="${STACK_DIR:-/opt/tchat}"
COMPOSE_FILE="$STACK_DIR/infra/compose/docker-compose.prod.yml"

cd "$STACK_DIR"
export TAG="${TAG:?falta TAG, el sha de la imagen a desplegar}"

echo "→ Desplegando $TAG"
docker compose -f "$COMPOSE_FILE" pull api migrate

echo "→ Aplicando migraciones"
docker compose -f "$COMPOSE_FILE" run --rm migrate

echo "→ Levantando servicios"
docker compose -f "$COMPOSE_FILE" up -d --remove-orphans

echo "→ Validando y recargando Caddy"
docker compose -f "$COMPOSE_FILE" exec -T caddy caddy validate --config /etc/caddy/Caddyfile
docker compose -f "$COMPOSE_FILE" exec -T caddy caddy reload --config /etc/caddy/Caddyfile

docker image prune -f >/dev/null

echo "✓ Despliegue completado"
