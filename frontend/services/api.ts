// Typed API client. Authentication rides on HttpOnly cookies set by the
// backend: requests use credentials: "include" and NO token is ever read
// from JavaScript. On a 401 we transparently try the refresh endpoint once.

import { apiPath } from "@/lib/config";

export class ApiError extends Error {
  status: number;
  code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

let refreshInFlight: Promise<boolean> | null = null;

async function tryRefresh(): Promise<boolean> {
  if (!refreshInFlight) {
    refreshInFlight = fetch(apiPath("/api/v1/auth/refresh"), {
      method: "POST",
      credentials: "include",
    })
      .then((res) => res.ok)
      .catch(() => false)
      .finally(() => {
        refreshInFlight = null;
      });
  }
  return refreshInFlight;
}

interface RequestOptions {
  method?: string;
  body?: unknown;
  skipRefresh?: boolean;
}

export async function api<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { method = "GET", body, skipRefresh = false } = options;

  async function perform(): Promise<Response> {
    return fetch(apiPath(path), {
      method,
      credentials: "include",
      headers: body !== undefined ? { "Content-Type": "application/json" } : undefined,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
  }

  let response = await perform();

  // Transparent single-flight refresh on expired access tokens.
  if (response.status === 401 && !skipRefresh && !path.startsWith("/api/v1/auth/")) {
    if (await tryRefresh()) {
      response = await perform();
    }
  }

  if (response.status === 204) {
    return undefined as T;
  }

  const contentType = response.headers.get("content-type") ?? "";
  if (!contentType.includes("application/json")) {
    if (!response.ok) {
      throw new ApiError(response.status, "http_error", `Request failed (${response.status})`);
    }
    return undefined as T;
  }

  const payload = await response.json();
  if (!response.ok) {
    const code = payload?.code ?? "error";
    const message = payload?.message ?? `Request failed (${response.status})`;
    throw new ApiError(response.status, code, message);
  }
  return payload as T;
}

/** Multipart upload helper (breakdown photos/videos). */
export async function apiUpload<T>(
  path: string,
  form: FormData,
): Promise<T> {
  const response = await fetch(apiPath(path), {
    method: "POST",
    credentials: "include",
    body: form,
  });
  if (!response.ok) {
    const payload = await response.json().catch(() => null);
    throw new ApiError(
      response.status,
      payload?.code ?? "upload_error",
      payload?.message ?? `Upload failed (${response.status})`,
    );
  }
  return (await response.json()) as T;
}
