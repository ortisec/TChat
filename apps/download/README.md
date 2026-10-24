# apps/download — descargas

Proyecto Astro para **`cli.tchat.site`**.

- Detecta el sistema operativo y ofrece el binario correcto.
- Sirve `install.sh`, `install.ps1` y `latest.json` (versión y checksums).
- Los binarios viven en GitHub Releases; esta app solo enlaza y verifica.
- Se despliega en Vercel con *Root Directory* `apps/download`.

**Estado:** pendiente (Paso 3). Los instaladores se generan con `cargo-dist` en el Paso 7.
