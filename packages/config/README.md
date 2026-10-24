# packages/config — configuración compartida

Bases reutilizables por las tres apps web.

| Archivo               | Qué contiene                                            |
| --------------------- | ------------------------------------------------------- |
| `tsconfig.base.json`  | `jsx: react-jsx` y comprobaciones estrictas adicionales |
| `eslint.config.mjs`   | Configuración plana: JS, TypeScript y Astro             |
| `prettier.config.mjs` | Prettier + `prettier-plugin-astro`                      |

## Uso

`tsconfig.json` de cada app:

```json
{
  "extends": ["astro/tsconfigs/strict", "../../packages/config/tsconfig.base.json"]
}
```

En la raíz, `eslint.config.mjs` y `prettier.config.mjs` reexportan la configuración de este paquete.
Los binarios de ESLint y Prettier viven en la raíz para tener una sola versión en todo el monorepo.

## Comandos

```powershell
pnpm lint
pnpm format
pnpm format:check
```

**Estado:** Paso 3 completado.
