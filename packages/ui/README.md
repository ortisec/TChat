# packages/ui — sistema de diseño

Componentes React y tokens compartidos por `apps/www`, `apps/account` y `apps/download`.

## Regla de color

Una única escala: **grises**. No se usan colores de acento. El énfasis se consigue con peso
tipográfico, bordes, opacidad y espaciado. Ver `docs/adr/0004-ui-tailwind-grayscale.md`.

## Componentes

| Export                                 | Uso                                                               |
| -------------------------------------- | ----------------------------------------------------------------- |
| `Button`                               | Primario, secundario y fantasma; renderiza `<a>` si recibe `href` |
| `Card`, `CardTitle`, `CardDescription` | Contenedores de contenido                                         |
| `Input`, `Label`, `FieldError`         | Formularios                                                       |
| `Badge`                                | Etiquetas monoespaciadas                                          |
| `Logo`                                 | Marca de TChat                                                    |
| `cn`                                   | Une clases condicionales                                          |

## Tokens

`src/styles/theme.css` define tipografía y radios, y la utilidad `container-page`.

## Consumo en Astro

Cada app importa el tema y declara las fuentes de Tailwind:

```css
@import "tailwindcss";
@import "../../../../packages/ui/src/styles/theme.css";

@source "../../../../packages/ui/src";
@source "../../src";
```

El `@source` es necesario porque Tailwind ignora `node_modules` por defecto y este paquete se
consume como código fuente del workspace.

**Estado:** Paso 3 completado.
