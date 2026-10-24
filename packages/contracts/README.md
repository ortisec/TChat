# packages/contracts — contratos

Fuente de verdad compartida entre backend, web y CLI.

- `openapi.json`: snapshot exportado por FastAPI. Se versiona y CI verifica que no haya deriva.
- `ws-events.md`: nombres, campos y direcciones de los eventos WebSocket (borrador hasta el Paso 4).

Flujo: FastAPI → `openapi.json` → `@tchat/api-client` → apps web.
El CLI implementa el mismo contrato con Serde y lo valida con tests propios.

## Regenerar

```powershell
just gen-client
```
