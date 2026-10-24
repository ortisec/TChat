#!/usr/bin/env bash
set -euo pipefail

STACK_DIR="${STACK_DIR:-/opt/tchat}"
COMPOSE_FILE="$STACK_DIR/infra/compose/docker-compose.prod.yml"
BACKUP_DIR="${BACKUP_DIR:-$STACK_DIR/backups}"
RETENTION_DAYS="${RETENTION_DAYS:-14}"

mkdir -p "$BACKUP_DIR"

timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
target="$BACKUP_DIR/tchat-$timestamp.dump.gz"

docker compose -f "$COMPOSE_FILE" exec -T postgres \
    sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" --format=custom' \
    | gzip -9 > "$target"

find "$BACKUP_DIR" -name 'tchat-*.dump.gz' -mtime +"$RETENTION_DAYS" -delete

echo "backup: $target ($(du -h "$target" | cut -f1))"
