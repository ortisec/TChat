# apps/cli — cliente de terminal

Workspace de Rust para el cliente de TChat.

```
crates/
├─ tchat-core/   # HTTP, WebSocket, auth, modelos, protocolo
├─ tchat-tui/    # Ratatui: pantallas, widgets, eventos
└─ tchat-cli/    # binario: args, config, keyring, bootstrap
```

- Se distribuye como binario para Windows, Linux y macOS vía GitHub Releases.
- `cargo-dist` genera instaladores y checksums a partir de un tag `cli-v*`.
- El refresh token se guarda en el keyring del sistema operativo.

**Estado:** pendiente (Paso 5). El release automatizado llega en el Paso 7.
