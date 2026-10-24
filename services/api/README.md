# services/api — backend

FastAPI + SQLAlchemy + PostgreSQL. Corre en el VPS con Docker Compose.

```
src/tchat_api/
├─ core/       # configuración, Firebase, tokens
├─ api/        # deps, serializers y routers REST
├─ realtime/   # WebSocket, presencia y protocolo
├─ models/     # SQLAlchemy
├─ schemas/    # Pydantic
├─ services/   # reglas de negocio
└─ db/         # motor, sesión y base declarativa
migrations/    # Alembic
tests/         # pytest + httpx
```

## Comandos

```powershell
just up        # PostgreSQL local en :5432
just migrate   # aplica migraciones
just api       # uvicorn en :8000 con recarga
just test      # pytest (crea sola la base tchat_test)
just check     # ruff + mypy
```

Documentación interactiva: <http://localhost:8000/docs>

## Endpoints

| Método    | Ruta                                   | Descripción                       |
| --------- | -------------------------------------- | --------------------------------- |
| GET       | `/health`                              | Liveness                          |
| GET       | `/health/ready`                        | Readiness (PostgreSQL)            |
| GET       | `/api/v1/meta`                         | Metadatos públicos                |
| POST      | `/api/v1/auth/sync`                    | Crea o actualiza el usuario       |
| POST      | `/api/v1/auth/device/start`            | Inicia el device flow             |
| POST      | `/api/v1/auth/device/approve`          | Aprueba el código desde la web    |
| POST      | `/api/v1/auth/device/token`            | Canjea el device code             |
| POST      | `/api/v1/auth/token`                   | Rota el refresh token             |
| POST      | `/api/v1/auth/logout`                  | Revoca el refresh token           |
| GET/PATCH | `/api/v1/users/me`                     | Perfil propio                     |
| GET       | `/api/v1/users/search?q=`              | Busca por username                |
| GET       | `/api/v1/users/{username}`             | Perfil público                    |
| GET       | `/api/v1/friends`                      | Lista de amigos                   |
| GET/POST  | `/api/v1/friends/requests`             | Solicitudes (entrantes/salientes) |
| POST      | `/api/v1/friends/requests/{id}/accept` | Acepta                            |
| POST      | `/api/v1/friends/requests/{id}/reject` | Rechaza                           |
| DELETE    | `/api/v1/friends/{user_id}`            | Elimina amistad                   |
| GET/POST  | `/api/v1/conversations`                | Lista y crea conversaciones       |
| GET       | `/api/v1/conversations/{id}`           | Detalle                           |
| GET/POST  | `/api/v1/conversations/{id}/messages`  | Historial y envío REST            |
| POST      | `/api/v1/conversations/{id}/read`      | Marca como leído                  |
| WS        | `/ws`                                  | Mensajería y presencia            |

## Autenticación

- **Web**: ID token de Firebase (`Authorization: Bearer …`).
- **CLI**: JWT propio emitido por `/auth/device/token`, 15 minutos, con refresh token opaco.
- La verificación de tokens de Firebase no requiere service account: se usan los certificados
  públicos de Google. `FIREBASE_SERVICE_ACCOUNT_JSON` solo hace falta para operaciones
  administrativas futuras.
- Los refresh tokens y device codes se guardan como SHA-256; nunca en claro.

## Tests

`pytest` usa `tchat_test`, la crea si no existe y reinicia las tablas por sesión. No necesita
Firebase: la identidad se inyecta con `app.dependency_overrides`. Cubre sync, device flow
(incluido expiración y `slow_down`), amistades, conversaciones, mensajes, lectura y protocolo WS.

## Contrato

```powershell
just gen-client
```

Exporta `packages/contracts/openapi.json` y regenera `packages/api-client`.

## Imagen Docker

```powershell
just image   # docker build -t tchat-api:local services/api

# pila completa en local:
docker compose -f infra/compose/docker-compose.yml --profile full up -d --build
```

La imagen es multi-stage: `uv` construye el entorno y la etapa final corre como usuario sin
privilegios con un healthcheck contra `/health`. Las migraciones se ejecutan con
`alembic upgrade head` en el contenedor `migrate` antes de desplegar.
