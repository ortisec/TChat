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
│  ├─ www/                    # tchat.site        · landing pública
│  ├─ account/                # app.tchat.site    · registro, login, perfil
│  ├─ download/               # cli.tchat.site    · descargas e instaladores
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

| Dominio          | Proyecto Vercel  | Contenido                                             |
| ---------------- | ---------------- | ----------------------------------------------------- |
| `tchat.site`     | `tchat-www`      | Landing, capturas, enlaces                            |
| `app.tchat.site` | `tchat-account`  | Registro, login, perfil, vinculación del CLI          |
| `cli.tchat.site` | `tchat-download` | Descargas, `install.sh`, `install.ps1`, `latest.json` |
| `api.tchat.site` | VPS (Caddy)      | API REST `/api/v1/*`                                  |
| `ws.tchat.site`  | VPS (Caddy)      | WebSocket `/ws`                                       |

Los binarios **no** se alojan en Vercel: viven en GitHub Releases y la página de descargas
enlaza a los assets y sirve los scripts de instalación.

## Autenticación

### Web

1. Astro renderiza la página; la isla React inicializa el SDK de Firebase.
2. El usuario se registra o inicia sesión (`email/password` y proveedores que se añadan después).
3. Firebase devuelve un ID token (1 h de vida).
4. La web llama a `POST /api/v1/auth/sync` con `Authorization: Bearer <id_token>` para crear o
   actualizar el registro en PostgreSQL.

### Terminal (device flow)

```
tchat login
   │  POST /api/v1/auth/device/start
   ▼
code + verification_url
   │  abre el navegador en app.tchat.site/cli-auth?code=…
   ▼
usuario inicia sesión con Firebase en la web
   │  la web confirma el código: POST /api/v1/auth/device/approve
   ▼
CLI hace polling GET /api/v1/auth/device/{code}
   │  recibe refresh token propio (opaco, revocable)
   ▼
lo guarda en el keyring del sistema operativo
```

El CLI pide ID tokens nuevos al backend (`POST /api/v1/auth/token`) usando su refresh token,
y los usa en el handshake del WebSocket.

### WebSocket

- URL: `wss://ws.tchat.site/ws`
- Handshake: `Authorization: Bearer <id_token>` (el CLI puede enviar cabeceras).
- El servidor valida el token con Firebase Admin, carga el usuario y registra la conexión.
- Cierre por expiración: el cliente reconecta con un token nuevo.

## Modelo de datos

| Tabla                  | Campos principales                                                          |
| ---------------------- | --------------------------------------------------------------------------- |
| `users`                | `id`, `firebase_uid`, `username`, `display_name`, `created_at`              |
| `friendships`          | `id`, `requester_id`, `addressee_id`, `status`, `created_at`, `accepted_at` |
| `conversations`        | `id`, `type` (direct/group), `created_at`                                   |
| `conversation_members` | `conversation_id`, `user_id`, `joined_at`                                   |
| `messages`             | `id`, `conversation_id`, `sender_id`, `body`, `created_at`, `edited_at`     |
| `device_tokens`        | `id`, `user_id`, `token_hash`, `name`, `last_used_at`, `revoked_at`         |

Los IDs son UUID v7 para ordenar por tiempo sin exponer secuencias.

## Puertos locales

| Servicio             | Puerto |
| -------------------- | ------ |
| `apps/www`           | 4321   |
| `apps/account`       | 4322   |
| `apps/download`      | 4323   |
| `services/api`       | 8000   |
| PostgreSQL (compose) | 5432   |

## Variables de entorno

| Ámbito                 | Variables                                                                                                                                                                        |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Web (públicas, Vercel) | `PUBLIC_FIREBASE_API_KEY`, `PUBLIC_FIREBASE_AUTH_DOMAIN`, `PUBLIC_FIREBASE_PROJECT_ID`, `PUBLIC_FIREBASE_APP_ID`, `PUBLIC_FIREBASE_SENDER_ID`, `PUBLIC_API_URL`, `PUBLIC_WS_URL` |
| Enlaces entre apps     | `PUBLIC_SITE_URL`, `PUBLIC_ACCOUNT_URL`, `PUBLIC_DOWNLOAD_URL`, `PUBLIC_RELEASES_URL`                                                                                            |
| API (privadas, VPS)    | `DATABASE_URL`, `FIREBASE_PROJECT_ID`, `FIREBASE_SERVICE_ACCOUNT_JSON`, `CORS_ORIGINS`, `TOKEN_TTL_SECONDS`                                                                      |
| CLI                    | ninguno obligatorio; la configuración vive en el keyring y en el directorio de configuración del usuario                                                                         |

Ningún secreto se versiona. Cada superficie incluye su propio `.env.example`.
