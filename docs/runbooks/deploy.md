# Despliegue (Vercel + VPS)

## Dominios

El apex `ortisec.site` está en Cloudflare y sirve tu portafolio. TChat no toca el apex ni `www`:
vive completo bajo `tchat.ortisec.site`.

| Dominio                  | Destino           | Contenido                                             |
| ------------------------ | ----------------- | ----------------------------------------------------- |
| `tchat.ortisec.site`     | Vercel `www`      | Landing, capturas, enlaces                            |
| `app.tchat.ortisec.site` | Vercel `account`  | Registro, login, perfil, vinculación del CLI          |
| `cli.tchat.ortisec.site` | Vercel `download` | Descargas, `install.sh`, `install.ps1`, `latest.json` |
| `api.tchat.ortisec.site` | VPS (Caddy)       | API REST `/api/v1/*`                                  |
| `ws.tchat.ortisec.site`  | VPS (Caddy)       | WebSocket `/ws`                                       |

### DNS en Cloudflare

Añade estos registros en la zona de `ortisec.site` (Cloudflare → DNS → Records):

| Nombre      | Tipo  | Contenido              | Proxy    |
| ----------- | ----- | ---------------------- | -------- |
| `tchat`     | CNAME | `cname.vercel-dns.com` | DNS only |
| `app.tchat` | CNAME | `cname.vercel-dns.com` | DNS only |
| `cli.tchat` | CNAME | `cname.vercel-dns.com` | DNS only |
| `api.tchat` | A     | `<IP del VPS>`         | DNS only |
| `ws.tchat`  | A     | `<IP del VPS>`         | DNS only |

Notas:

- Usa el valor exacto que muestre Vercel al añadir cada dominio; puede variar entre
  `cname.vercel-dns.com` y una IP anycast.
- **Desactiva el proxy naranja** en estos subdominios: el certificado universal de Cloudflare no
  cubre `*.tchat.ortisec.site` y la validación de Vercel/Let's Encrypt falla.
- Si Vercel pide un registro `TXT` de verificación, añádelo también en Cloudflare (DNS only).
- No modifiques los registros del apex ni de `www` que usa el portafolio.

## Web en Vercel

Crea tres proyectos desde el mismo repositorio (`ortisec/TChat`), cada uno con su _Root
Directory_. Framework **Astro**, branch `main`, y activa **Include files outside the root
directory** para que pnpm resuelva `packages/ui` y `packages/config`.

| Proyecto         | Root Directory  | Dominio                  |
| ---------------- | --------------- | ------------------------ |
| `tchat-www`      | `apps/www`      | `tchat.ortisec.site`     |
| `tchat-account`  | `apps/account`  | `app.tchat.ortisec.site` |
| `tchat-download` | `apps/download` | `cli.tchat.ortisec.site` |

> **Error típico:** `No Output Directory named "dist" found`. Significa que el proyecto está
> construyendo desde la raíz del monorepo: el build deja `apps/www/dist` y Vercel busca `dist/`.
> Corrige **Root Directory** a `apps/www` (o el de la app), marca el preset **Astro** y deja
> **Output Directory** en `dist`. Después, _Redeploy_ sin caché. Los `vercel.json` de cada app
> fijan `outputDirectory` para evitarlo.

Variables de entorno (Production y Preview):

```env
# tchat-www
PUBLIC_SITE_URL=https://tchat.ortisec.site
PUBLIC_ACCOUNT_URL=https://app.tchat.ortisec.site
PUBLIC_DOWNLOAD_URL=https://cli.tchat.ortisec.site
```

```env
# tchat-account
PUBLIC_SITE_URL=https://tchat.ortisec.site
PUBLIC_API_URL=https://api.tchat.ortisec.site
PUBLIC_FIREBASE_API_KEY=<el de apps/account/.env>
PUBLIC_FIREBASE_AUTH_DOMAIN=tchat-c20ce.firebaseapp.com
PUBLIC_FIREBASE_PROJECT_ID=tchat-c20ce
PUBLIC_FIREBASE_APP_ID=<el de apps/account/.env>
PUBLIC_FIREBASE_SENDER_ID=256761907628
```

```env
# tchat-download
PUBLIC_SITE_URL=https://tchat.ortisec.site
PUBLIC_ACCOUNT_URL=https://app.tchat.ortisec.site
PUBLIC_RELEASES_URL=https://github.com/ortisec/TChat/releases/latest
```

Después:

1. Asigna los dominios en Vercel y añade los registros DNS de la tabla.
2. Firebase Console → Authentication → Settings → **Authorized domains** →
   `app.tchat.ortisec.site`.
3. Redeploy de los tres proyectos: Astro incrusta las variables `PUBLIC_*` en el build.
4. Verifica `https://tchat.ortisec.site`, `https://app.tchat.ortisec.site` y
   `https://cli.tchat.ortisec.site`.

## Cómo queda el VPS

```
Internet
   │ 80/443
   ▼
 Caddy  ──►  api:8000  ──►  postgres:5432
 (TLS)      (FastAPI)       (volumen)
```

- Caddy termina TLS con Let's Encrypt para `api.tchat.ortisec.site` y `ws.tchat.ortisec.site`.
- La API y PostgreSQL solo existen en la red interna de Docker; PostgreSQL no publica puertos.
- Las migraciones se ejecutan en un contenedor efímero (`migrate`) antes de levantar la versión.
- Las imágenes viven en GHCR con el SHA del commit como tag.

## Preparación del VPS (una sola vez)

1. **Servidor**: Ubuntu 24.04 con Docker Engine y el plugin `docker compose`.
2. **DNS**: registros `A` de `api.tchat.ortisec.site` y `ws.tchat.ortisec.site` apuntando a la IP
   del VPS (DNS only en Cloudflare).
3. **Firewall**: permitir 22 (SSH), 80 y 443 (TCP y UDP para HTTP/3).
4. **Usuario de despliegue**: debe poder ejecutar Docker (`usermod -aG docker <usuario>`).
5. **Directorio**: `/opt/tchat`, propiedad del usuario SSH (el workflow lo sincroniza por `tar`).
6. **Variables**: copia `infra/compose/.env.example` a `/opt/tchat/infra/compose/.env` y rellena:
   - `POSTGRES_PASSWORD` y `AUTH_JWT_SECRET` con valores aleatorios.
   - `ACME_EMAIL` con tu correo real.
   - `CORS_ORIGINS` y `PUBLIC_ACCOUNT_URL` con los dominios de producción:
     `https://tchat.ortisec.site,https://app.tchat.ortisec.site,https://cli.tchat.ortisec.site`
     y `https://app.tchat.ortisec.site`.
7. **GitHub → Settings → Environments → `production`**, secretos:
   - `VPS_HOST`: IP o dominio.
   - `VPS_USER`: usuario SSH.
   - `VPS_SSH_KEY`: clave privada ed25519 completa.
   - `VPS_PORT`: opcional (22 por defecto).
8. **Clave SSH**: añade la pública a `~/.ssh/authorized_keys` del VPS.
9. **GHCR**: el workflow se autentica con `GITHUB_TOKEN`. Para hacer `pull` manual fuera del
   workflow necesitas un PAT con `read:packages` o hacer público el paquete.

## Despliegue de la API

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
curl -fsS https://api.tchat.ortisec.site/health
curl -fsS https://api.tchat.ortisec.site/health/ready
curl -fsS https://api.tchat.ortisec.site/api/v1/meta

# WebSocket (debe responder 403 sin token, no 502)
curl -i https://ws.tchat.ortisec.site/ws
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

| Síntoma                      | Causa probable                                                        |
| ---------------------------- | --------------------------------------------------------------------- |
| Caddy no obtiene certificado | DNS mal apuntado, proxy naranja de Cloudflare o puerto 80/443 cerrado |
| `ready` devuelve 503         | PostgreSQL caído o `DATABASE_URL` incorrecta                          |
| 502 en `api.`                | Contenedor `api` reiniciándose: revisa logs                           |
| WS 403                       | Correcto sin token; el CLI debe enviar `Authorization`                |
| El deploy falla en `migrate` | Migración incompatible: revisa logs y restaura backup                 |

## Mantenimiento

- **Actualizar imágenes base**: Dependabot ya vigila las acciones; las imágenes (`python`,
  `postgres`, `caddy`) se actualizan a mano en el Dockerfile/compose.
- **Disco**: `docker image prune -f` corre en cada despliegue; revisa `df -h` periódicamente.
- **Backups**: ver `backup-restore.md`.
