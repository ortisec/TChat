import { createApiClient } from "@tchat/api-client";

export const apiBaseUrl = import.meta.env.PUBLIC_API_URL ?? "http://localhost:8000";

export const api = createApiClient(apiBaseUrl);

/** Extrae el `detail` que devuelve FastAPI en los errores. */
export function detailFrom(error: unknown): string {
  if (typeof error === "object" && error !== null && "detail" in error) {
    const detail = (error as { detail: unknown }).detail;
    if (typeof detail === "string") {
      return detail;
    }
  }
  return "";
}
