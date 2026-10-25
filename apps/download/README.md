# apps/download — descargas

Proyecto Astro + React para **`cli.tchat.ortisec.site`**.

- Detecta el sistema operativo del visitante en el navegador.
- Lee `/latest.json` para ofrecer los artefactos de la última release, agrupados por plataforma.
- Enlaza directamente a los assets de GitHub Releases (los binarios no se alojan en Vercel).
- Rewrites de Vercel para servir desde este dominio:
  - `/install.sh` → instalador oficial generado por `cargo-dist`.
  - `/install.ps1` → instalador de PowerShell.
  - `/latest.json` → metadatos de la última release.

## latest.json

```json
{
  "version": "0.1.0",
  "tag": "tchat-cli-v0.1.0",
  "released_at": "2026-10-24T18:00:00Z",
  "assets": [
    {
      "name": "tchat-cli-x86_64-pc-windows-msvc.zip",
      "url": "https://github.com/ortisec/TChat/releases/download/tchat-cli-v0.1.0/…",
      "size": 1234567,
      "sha256": "…"
    }
  ]
}
```

Lo genera `infra/scripts/latest-json.sh` tras publicar la release.

## Desarrollo

```powershell
pnpm --filter @tchat/download dev   # http://localhost:4323
```

Sin `latest.json` (no hay release publicada) las tarjetas enlazan a GitHub Releases.

**Estado:** Paso 7 completado.
