#!/usr/bin/env bash
set -euo pipefail

FILE="${1:?uso: restore.sh <archivo.dump.gz>}"
STACK_DIR="${STACK_DIR:-/opt/tchat}"
COMPOSE_FILE="$STACK_DIR/infra/compose/docker-compose.prod.yml"

if [ ! -f "$FILE" ]; then
    echo "No existe el archivo: $FILE" >&2
    exit 1
fi

echo "⚠  Esto REEMPLAZA por completo la base de datos de TChat."
read -r -p "Escribe 'restaurar' para continuar: " answer
if [ "$answer" != "restaurar" ]; then
    echo "Cancelado."
    exit 1
fi

gunzip -c "$FILE" | docker compose -f "$COMPOSE_FILE" exec -T postgres \
    sh -c 'pg_restore -U "$POSTGRES_USER" -d "$POSTGRES_DB" --clean --if-exists --no-owner'

echo "✓ Restauración completada"
