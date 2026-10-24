# Despliegue de la API

## Cómo queda el VPS

```
Internet
   │ 80/443
   ▼
 Caddy  ──►  api:8000  ──►  postgres:5432
 (TLS)      (FastAPI)       (volumen)
```

- Caddy termina TLS con Let's Encrypt para `api.tchat.site` y `ws.tchat.site`.
- La API y PostgreSQL solo existen en la red interna de Docker; PostgreSQL no publica puertos.
- Las migraciones se ejecutan en un contenedor efímero (`migrate`) antes de levantar la versión.
- Las imágenes viven en GHCR con el SHA del commit como tag.

## Preparación del VPS (una sola vez)

1. **Servidor**: Ubuntu 24.04 con Docker Engine y el plugin `docker compose`.
2. **DNS**: registros `A` de `api.tchat.site` y `ws.tchat.site` apuntando a la IP del VPS.
3. **Firewall**: permitir 22 (SSH), 80 y 443 (TCP y UDP para HTTP/3).
4. **Usuario de despliegue**: debe poder ejecutar Docker (`usermod -aG docker <usuario>`).
5. **Directorio**: `/opt/tchat` (el workflow lo crea y sincroniza `infra/`).
6. **Variables**: copia `infra/compose/.env.example` a `/opt/tchat/infra/compose/.env` y rellena:
   - `POSTGRES_PASSWORD` y `AUTH_JWT_SECRET` con valores aleatorios.
   - `ACME_EMAIL` con tu correo real.
   - `CORS_ORIGINS` y `PUBLIC_ACCOUNT_URL` con los dominios de producción.
7. **GitHub → Settings → Environments → `production`**, secretos:
   - `VPS_HOST`: IP o dominio.
   - `VPS_USER`: usuario SSH.
   - `VPS_SSH_KEY`: clave privada ed25519 completa.
   - `VPS_PORT`: opcional (22 por defecto).
8. **Clave SSH**: añade la pública a `~/.ssh/authorized_keys` del VPS.
9. **GHCR**: el workflow se autentica con `GITHUB_TOKEN`. Para hacer `pull` manual fuera del
   workflow necesitas un PAT con `read:packages` o hacer público el paquete.

## Despliegue

Automático: cualquier push a `main` que toque `services/api/**` o `infra/**`.

Manual: **Actions → Deploy API → Run workflow**.

El workflow:

1. Construye la imagen y la publica como `ghcr.io/ortisec/tchat-api:<sha>` y `:latest`.
2. Prepara la llave SSH y autentica Docker contra GHCR en el VPS.
3. Sincroniza `infra/` por `tar` a `/opt/tchat` (no toca el `.env`).
4. Ejecuta `deploy.sh` con `TAG=<sha>`:
   - `docker compose pull`
   - `run --rm migrate` (`alembic upgrade head`)
   - `up -d --remove-orphans`
   - `caddy validate` + `caddy reload`
   - limpia imágenes huérfanas

## Verificación

```bash
curl -fsS https://api.tchat.site/health
curl -fsS https://api.tchat.site/health/ready
curl -fsS https://api.tchat.site/api/v1/meta

# WebSocket (debe responder 403 sin token, no 502)
curl -i https://ws.tchat.site/ws
```

Y desde el cliente: `tchat status` y `tchat`.

## Rollback

En el VPS, con una imagen anterior todavía en GHCR:

```bash
cd /opt/tchat
TAG=<sha-anterior> bash infra/scripts/deploy.sh
```

Las migraciones no se revierten automáticamente: si el cambio incluye una migración incompatible,
restaura el backup previo (ver `backup-restore.md`) antes de volver a la versión anterior.

## Diagnóstico

```bash
cd /opt/tchat
docker compose -f infra/compose/docker-compose.prod.yml ps
docker compose -f infra/compose/docker-compose.prod.yml logs -f api
docker compose -f infra/compose/docker-compose.prod.yml logs -f caddy
docker compose -f infra/compose/docker-compose.prod.yml exec api alembic current
```

| Síntoma                      | Causa probable                                         |
| ---------------------------- | ------------------------------------------------------ |
| Caddy no obtiene certificado | DNS mal apuntado o puerto 80/443 cerrado               |
| `ready` devuelve 503         | PostgreSQL caído o `DATABASE_URL` incorrecta           |
| 502 en `api.`                | Contenedor `api` reiniciándose: revisa logs            |
| WS 403                       | Correcto sin token; el CLI debe enviar `Authorization` |
| El deploy falla en `migrate` | Migración incompatible: revisa logs y restaura backup  |

## Mantenimiento

- **Actualizar imágenes base**: Dependabot ya vigila las acciones; las imágenes (`python`,
  `postgres`, `caddy`) se actualizan a mano en el Dockerfile/compose.
- **Disco**: `docker image prune -f` corre en cada despliegue; revisa `df -h` periódicamente.
- **Backups**: ver `backup-restore.md`.
