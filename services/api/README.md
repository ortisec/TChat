# services/api — backend

FastAPI + SQLAlchemy + PostgreSQL. Corre en el VPS con Docker Compose.

```
src/tchat_api/
├─ core/       # configuración, seguridad, logging
├─ api/        # health y routers REST
├─ realtime/   # WebSocket y presencia (Paso 4)
├─ models/     # SQLAlchemy
├─ schemas/    # Pydantic
├─ services/   # casos de uso
└─ db/         # motor, sesión y base declarativa
migrations/    # Alembic
tests/         # pytest + httpx
```

## Comandos

```powershell
just up        # PostgreSQL local en :5432
just migrate   # aplica migraciones
just api       # uvicorn en :8000 con recarga
just test      # pytest
just check     # ruff + mypy
```

Documentación interactiva: <http://localhost:8000/docs>

## Endpoints del Paso 2

| Método | Ruta            | Descripción                                  |
| ------ | --------------- | -------------------------------------------- |
| GET    | `/health`       | Liveness (no toca la base de datos)          |
| GET    | `/health/ready` | Readiness (comprueba PostgreSQL)             |
| GET    | `/api/v1/meta`  | Nombre, versión, entorno y proyecto Firebase |

## Contrato

```powershell
just gen-client
```

Esto exporta `packages/contracts/openapi.json` y regenera `packages/api-client`.
La API es la única fuente de verdad; no se editan tipos a mano.
