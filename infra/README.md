# infra — despliegue y operación

```
compose/
├─ docker-compose.yml        # desarrollo (PostgreSQL; API opcional con --profile full)
├─ docker-compose.prod.yml   # VPS (Caddy + API + PostgreSQL + migrate)
└─ .env.example              # variables para el VPS
caddy/
└─ Caddyfile                 # TLS y proxy para api. y ws.
scripts/
├─ deploy.sh                 # pull + migraciones + up + recarga de Caddy
├─ backup.sh                 # pg_dump comprimido con retención
└─ restore.sh                # restauración con confirmación
```

Reglas:

- Las imágenes de la API se publican en GHCR y el VPS solo hace `pull`.
- Las migraciones se ejecutan antes de levantar la nueva versión.
- PostgreSQL nunca expone su puerto fuera de la red de Docker.
- Los secretos se inyectan por `.env` en el VPS, nunca en la imagen ni en el repositorio.

## Desarrollo

```powershell
just up        # solo PostgreSQL
just api       # API local con uv

# o la pila completa en contenedores:
docker compose -f infra/compose/docker-compose.yml --profile full up -d --build
```

## Producción

Ver `docs/runbooks/deploy.md` y `docs/runbooks/backup-restore.md`.
