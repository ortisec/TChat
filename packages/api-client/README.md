# packages/api-client — cliente tipado

Generado desde `packages/contracts/openapi.json` con `openapi-typescript` y `openapi-fetch`.

- **No se edita a mano.** Cualquier cambio se pierde en la siguiente generación.
- Se versiona en git para que la web compile sin necesitar Python.
- CI verifica que esté sincronizado con el OpenAPI real.

## Uso

```ts
import { createApiClient } from "@tchat/api-client";

const api = createApiClient("http://localhost:8000");

const { data, error } = await api.GET("/api/v1/meta");
```

## Regenerar

```powershell
just gen-client
```

No requiere PostgreSQL: solo importa la app para leer el esquema.
