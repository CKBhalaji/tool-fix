// Public, frontend-safe configuration only.
// NOTE: Firebase values are deliberately absent — Firebase is a
// backend-only dependency; the browser never sees it.

export const API_URL =
  process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

export const WS_URL =
  process.env.NEXT_PUBLIC_WS_URL ?? API_URL.replace(/^http/, "ws");

export function apiPath(path: string): string {
  return `${API_URL}${path}`;
}
