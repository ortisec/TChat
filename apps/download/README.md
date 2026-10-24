# apps/download — descargas

Proyecto Astro + React para **`cli.tchat.site`**.

- Detecta el sistema operativo del visitante en el navegador y resalta su plataforma.
- Tarjetas para Windows, macOS y Linux con el nombre de artefacto que publicará `cargo-dist`.
- Enlaces a GitHub Releases: los binarios no se alojan en Vercel.
- Comandos de instalación previstos (`install.sh` / `install.ps1`).

## Desarrollo

```powershell
pnpm --filter @tchat/download dev   # http://localhost:4323
```

## Variables

| Variable              | Uso                                                 |
| --------------------- | --------------------------------------------------- |
| `PUBLIC_RELEASES_URL` | Página de releases (por defecto la del repositorio) |

**Estado:** Paso 3 completado. Los binarios e instaladores reales se generan en el Paso 7.
