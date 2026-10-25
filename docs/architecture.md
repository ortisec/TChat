# Arquitectura de TChat

> Diagrama de referencia: `docs/assets/arquitectura.png`.

TChat se compone de tres runtimes independientes que se comunican por HTTPS y WebSocket:

- **Cliente web** (Astro + React) desplegado en Vercel, en tres proyectos/dominios.
- **Cliente de terminal** (Rust + Ratatui + Tokio) distribuido como binario.
- **Backend** (FastAPI + SQLAlchemy + PostgreSQL) en un VPS con Docker Compose.

La identidad la gestiona **Firebase Authentication**. El backend nunca almacena contraseñas: valida
los ID tokens con el Firebase Admin SDK y guarda únicamente `firebase_uid` como vínculo.

## Estructura del monorepo

```
tchat/
├─ apps/
│  ├─ www/                    # tchat.ortisec.site        · landing pública
│  ├─ account/                # app.tchat.ortisec.site    · registro, login, perfil
│  ├─ download/               # cli.tchat.ortisec.site    · descargas e instaladores
│  └─ cli/                    # Rust workspace
│     └─ crates/
│        ├─ tchat-core/       # HTTP, WebSocket, auth, modelos, protocolo
│        ├─ tchat-tui/        # Ratatui: pantallas, widgets, eventos
│        └─ tchat-cli/        # binario: args, config, keyring, bootstrap
├─ services/
│  └─ api/
│     ├─ src/tchat_api/
│     │  ├─ core/             # config, seguridad, logging
│     │  ├─ api/v1/           # routers REST
│     │  ├─ realtime/         # WebSocket y presencia
│     │  ├─ models/           # SQLAlchemy
│     │  ├─ schemas/          # Pydantic
│     │  ├─ services/         # casos de uso
│     │  └─ db/               # sesión y base declarativa
│     ├─ migrations/          # Alembic
│     └─ tests/
├─ packages/
│  ├─ ui/                     # componentes React + tokens de diseño
│  ├─ api-client/             # generado desde OpenAPI (no editar a mano)
│  ├─ config/                 # tsconfig, eslint, prettier compartidos
│  └─ contracts/              # openapi.json + protocolo WebSocket
├─ infra/
│  ├─ compose/                # docker-compose dev y prod
│  ├─ caddy/                  # Caddyfile
│  └─ scripts/                # install.sh, install.ps1, backup.sh, deploy.sh
├─ docs/
│  ├─ adr/                    # decisiones registradas
│  └─ assets/                 # diagramas
├─ .github/workflows/         # CI y despliegue
├─ justfile                   # tareas del monorepo
├─ package.json               # raíz del workspace JS (delgada)
└─ pnpm-workspace.yaml
```

## Reglas de dependencia

```
apps/www      ─┬─> packages/ui
apps/account  ─┼─> packages/api-client
apps/download ─┘

apps/cli      ───> services/api (solo por red, nunca por código)
services/api  ───> packages/contracts/openapi.json (verifica, no importa)
```

Reglas estrictas:

1. Ningún paquete de `packages/` importa código de `apps/` ni de `services/`.
2. `services/api` no depende de nada JS. Su contrato público es HTTP.
3. `packages/api-client` es **generado**: cualquier cambio manual se sobrescribe.
4. La raíz no contiene manifiestos de Python ni de Rust (solo JS y `just`).

## Contrato primero

```
FastAPI (schemas Pydantic)
        │  uv run python -m tchat_api.export_openapi
        ▼
packages/contracts/openapi.json      ← se versiona en git
        │  pnpm gen:client
        ▼
packages/api-client/src/…            ← se versiona en git
        │
        ▼
apps/account, apps/www, apps/download
```

CI compara el snapshot con el OpenAPI real: si difieren, el PR falla con el mensaje
"regenera el contrato y el cliente". Esto evita que la web y el backend se desincronicen.

El protocolo WebSocket se documenta en `packages/contracts/ws-events.md` y se implementa
dos veces: Pydantic (backend) y Serde (Rust). Los nombres de eventos son el contrato.

## Subdominios y despliegue

El apex `ortisec.site` está reservado al portafolio; TChat vive completo bajo
`tchat.ortisec.site` y sus subdominios.

| Dominio                  | Proyecto Vercel  | Contenido                                             |
| ------------------------ | ---------------- | ----------------------------------------------------- |
| `tchat.ortisec.site`     | `tchat-www`      | Landing, capturas, enlaces                            |
| `app.tchat.ortisec.site` | `tchat-account`  | Registro, login, perfil, vinculación del CLI          |
| `cli.tchat.ortisec.site` | `tchat-download` | Descargas, `install.sh`, `install.ps1`, `latest.json` |
| `api.tchat.ortisec.site` | VPS (Caddy)      | API REST `/api/v1/*`                                  |
| `ws.tchat.ortisec.site`  | VPS (Caddy)      | WebSocket `/ws`                                       |

Los binarios **no** se alojan en Vercel: viven en GitHub Releases y la página de descargas
enlaza a los assets y sirve los scripts de instalación.

### Backend en el VPS

```
Internet ──80/443──► Caddy ──► api:8000 ──► postgres:5432
                     (TLS)     (FastAPI)     (volumen)
```

- Las imágenes se construyen en GitHub Actions y se publican en GHCR con el SHA del commit.
- `deploy-api.yml` sincroniza `infra/`, hace `pull`, aplica migraciones con un contenedor
  efímero (`migrate`), levanta los servicios y recarga Caddy.
- PostgreSQL no publica puertos; solo es accesible desde la red de Docker.
- El `.env` de producción vive únicamente en `/opt/tchat/infra/compose/.env`.
- Los backups son `pg_dump` custom comprimidos, con retención de 14 días por defecto.

Runbooks: `docs/runbooks/deploy.md` y `docs/runbooks/backup-restore.md`.

## Autenticación

### Web

1. Astro renderiza la página; la isla React inicializa el SDK de Firebase.
2. El usuario se registra o inicia sesión (`email/password` y proveedores que se añadan después).
3. Firebase devuelve un ID token (1 h de vida).
4. La web llama a `POST /api/v1/auth/sync` con `Authorization: Bearer <id_token>` para crear o
   actualizar el registro en PostgreSQL.

### Terminal (device flow)

1. `POST /api/v1/auth/device/start` devuelve `user_code`, `device_code`, `verification_url`,
   `expires_in` e `interval`.
   - `user_code`: 8 caracteres que el usuario ve en la web y en el terminal.
   - `device_code`: secreto que solo conserva el CLI; el servidor guarda su hash SHA-256.
2. `tchat login` abre `verification_url` (`app.tchat.ortisec.site/cli-auth?code=USER_CODE`).
3. El usuario inicia sesión y pulsa «Autorizar terminal»: `POST /api/v1/auth/device/approve`
   con su ID token de Firebase y el `user_code`.
4. El CLI hace polling a `POST /api/v1/auth/device/token` con el `device_code` y puede recibir
   `authorization_pending`, `slow_down`, `expired_token` o `invalid_grant`.
5. Al aprobarse recibe `access_token` (JWT propio, 15 min) y `refresh_token` (opaco, revocable).
6. `POST /api/v1/auth/token` rota el refresh token y emite un acceso nuevo;
   `POST /api/v1/auth/logout` lo revoca.

Los códigos caducan en 10 minutos y solo se canjean una vez. El CLI guarda el refresh token en el
keyring del sistema operativo.

### WebSocket

- URL: `wss://ws.tchat.ortisec.site/ws`
- Handshake: `Authorization: Bearer <token>`. Acepta el JWT del CLI y los ID tokens de Firebase.
- Cliente → servidor: `ping`, `message.send`, `message.read`, `typing.start`, `typing.stop`.
- Servidor → cliente: `pong`, `message.new`, `message.ack`, `presence.update`, `typing.update`,
  `error`.
- Cada mensaje se entrega a todos los miembros conectados de la conversación.
- Presencia: `online` al abrir la primera conexión del usuario y `offline` al cerrar la última.
- El registro de conexiones vive en memoria: para escalar a varias instancias hay que moverlo a
  Redis con pub/sub.

## Cliente de terminal

- Workspace Rust 2024 con tres crates: `tchat-core` (red y protocolo), `tchat-tui` (estado y
  dibujo) y `tchat-cli` (binario y comandos).
- Comandos: `tchat login`, `logout`, `status`, `open <usuario>` y la TUI por defecto.
- El access token dura 15 minutos y se renueva con el refresh token antes de cada reconexión.
- El refresh token vive en el llavero nativo (Credential Manager, Keychain o keyutils).
- La TUI mantiene un WebSocket con reconexión y backoff; los mensajes del usuario se envían por
  WebSocket y el historial se carga por REST.
- Compatible con Windows, Linux y macOS; el empaquetado con `cargo-dist` se publica al empujar un
  tag `tchat-cli-v*` (Paso 7).

### Release del cliente

- `cargo-dist` vive en `apps/cli` y publica al empujar un tag `tchat-cli-v*`.
- Targets: Windows x86_64 (zip + MSI), macOS Intel y Apple Silicon, Linux x86_64 glibc y musl.
- Instaladores `install.sh` / `install.ps1`, checksums SHA-256 y `latest.json`.
- `cli.tchat.ortisec.site` sirve los instaladores y los metadatos mediante rewrites de Vercel hacia
  GitHub Releases: los binarios no pasan por Vercel.
- La página de descargas lee `/latest.json` para mostrar versión y enlaces reales.

## Superficie HTTP

| Grupo          | Rutas                                                                                                                          |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Salud          | `GET /health`, `GET /health/ready`                                                                                             |
| Meta           | `GET /api/v1/meta`                                                                                                             |
| Auth           | `POST /api/v1/auth/sync`, `/device/start`, `/device/approve`, `/device/token`, `/token`, `/logout`                             |
| Usuarios       | `GET/PATCH /api/v1/users/me`, `GET /users/search`, `GET /users/{username}`                                                     |
| Amistades      | `GET /friends`, `GET/POST /friends/requests`, `POST /friends/requests/{id}/accept` y `/reject`, `DELETE /friends/{user_id}`    |
| Conversaciones | `GET/POST /conversations`, `GET /conversations/{id}`, `GET/POST /conversations/{id}/messages`, `POST /conversations/{id}/read` |

## Modelo de datos

| Tabla                  | Campos principales                                                                      |
| ---------------------- | --------------------------------------------------------------------------------------- |
| `users`                | `id`, `firebase_uid`, `username`, `email`, `display_name`, `created_at`                 |
| `friendships`          | `id`, `requester_id`, `addressee_id`, `status`, `created_at`, `responded_at`            |
| `conversations`        | `id`, `type` (direct/group), `title`, `created_by`, `updated_at`                        |
| `conversation_members` | `conversation_id`, `user_id`, `joined_at`, `last_read_at`                               |
| `messages`             | `id`, `conversation_id`, `sender_id`, `body`, `created_at`, `edited_at`                 |
| `device_tokens`        | `id`, `user_id`, `token_hash`, `name`, `last_used_at`, `revoked_at`                     |
| `device_codes`         | `id`, `user_code`, `device_code_hash`, `status`, `user_id`, `expires_at`, `approved_at` |

Los IDs son UUID v7 monótonos: ordenan por tiempo incluso dentro del mismo milisegundo. El refresh
token y el device code nunca se guardan en claro, solo su SHA-256.

## Puertos locales

| Servicio             | Puerto |
| -------------------- | ------ |
| `apps/www`           | 4321   |
| `apps/account`       | 4322   |
| `apps/download`      | 4323   |
| `services/api`       | 8000   |
| PostgreSQL (compose) | 5432   |

## Variables de entorno

| Ámbito                 | Variables                                                                                                                                                                                                                             |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Web (públicas, Vercel) | `PUBLIC_FIREBASE_API_KEY`, `PUBLIC_FIREBASE_AUTH_DOMAIN`, `PUBLIC_FIREBASE_PROJECT_ID`, `PUBLIC_FIREBASE_APP_ID`, `PUBLIC_FIREBASE_SENDER_ID`, `PUBLIC_API_URL`                                                                       |
| Enlaces entre apps     | `PUBLIC_SITE_URL`, `PUBLIC_ACCOUNT_URL`, `PUBLIC_DOWNLOAD_URL`, `PUBLIC_RELEASES_URL`                                                                                                                                                 |
| API (privadas, VPS)    | `DATABASE_URL`, `FIREBASE_PROJECT_ID`, `FIREBASE_SERVICE_ACCOUNT_JSON`, `CORS_ORIGINS`, `AUTH_JWT_SECRET`, `ACCESS_TOKEN_TTL_SECONDS`, `DEVICE_CODE_TTL_SECONDS`, `DEVICE_POLL_INTERVAL_SECONDS`, `PUBLIC_ACCOUNT_URL`, `ENVIRONMENT` |
| Infra (VPS)            | `TAG`, `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`, `ACME_EMAIL`                                                                                                                                                              |
| CLI                    | ninguno obligatorio; `TCHAT_API_URL`, `TCHAT_WS_URL` y `TCHAT_ACCOUNT_URL` permiten apuntar a otro entorno. El refresh token vive en el keyring                                                                                       |

Ningún secreto se versiona. Cada superficie incluye su propio `.env.example`.
