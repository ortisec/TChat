# 0004 — Tailwind con escala de grises

- **Estado:** aceptado
- **Fecha:** 2026-10-24

## Contexto

La interfaz web debe ser sobria, legible en terminal y coherente con la marca. Se quiere avanzar
rápido sin diseñar un sistema de color complejo.

## Decisión

- Tailwind CSS en las tres apps web.
- Una sola paleta: escala de grises (`neutral`), sin colores de acento.
- Los tokens viven en `packages/ui` y se exponen como variables CSS; las apps no definen colores.
- El énfasis se consigue con peso tipográfico, bordes, opacidad y espaciado, no con color.
- Modo claro y oscuro derivados de la misma escala, no de paletas distintas.

## Consecuencias

- Interfaz consistente entre landing, cuenta y descargas.
- Los estados (error, éxito, aviso) se comunican con iconografía y texto, no con color:
  hay que ser explícito en los mensajes.
- Añadir un color de acento más adelante es una decisión con su propio ADR.

## Alternativas descartadas

- **Sistema de diseño propio desde cero:** coste alto antes de tener producto.
- **Bootstrap u otro framework con tema:** impone colores y estilos difíciles de reducir a grises.
