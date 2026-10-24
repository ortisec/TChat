# apps/www — landing

Proyecto Astro + React para **`tchat.site`**.

- Landing pública: propuesta de valor, mockup de terminal, características y pasos de uso.
- No carga Firebase ni el cliente de API.
- Usa `@tchat/ui` y Tailwind en escala de grises.

## Desarrollo

```powershell
pnpm --filter @tchat/www dev   # http://localhost:4321
pnpm --filter @tchat/www build
```

Recuerda que `just dev` arranca las tres apps a la vez.

## Variables

| Variable              | Uso                                                              |
| --------------------- | ---------------------------------------------------------------- |
| `PUBLIC_ACCOUNT_URL`  | Enlaces a `app.tchat.site` (por defecto `http://localhost:4322`) |
| `PUBLIC_DOWNLOAD_URL` | Enlaces a `cli.tchat.site` (por defecto `http://localhost:4323`) |

**Estado:** Paso 3 completado.
