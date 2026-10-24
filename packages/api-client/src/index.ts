import createClient from "openapi-fetch";

import type { paths } from "./schema";

export type { components, operations, paths } from "./schema";

/**
 * Crea un cliente tipado contra el contrato OpenAPI de TChat.
 *
 * @param baseUrl URL base de la API, por ejemplo `https://api.tchat.site`.
 * @param fetchImpl implementación de `fetch` (útil para inyectar cookies o tests).
 */
export function createApiClient(baseUrl: string, fetchImpl?: typeof fetch) {
  return fetchImpl
    ? createClient<paths>({ baseUrl, fetch: fetchImpl })
    : createClient<paths>({ baseUrl });
}
