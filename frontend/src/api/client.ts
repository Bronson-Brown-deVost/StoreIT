import type { ErrorResponse } from "./types";
import { queueMutation } from "~/lib/offlineQueue";

const BASE = "/api/v1";

export class ApiClientError extends Error {
  constructor(
    public code: string,
    message: string,
    public status: number,
  ) {
    super(message);
    this.name = "ApiClientError";
  }
}

async function handleResponse<T>(res: Response): Promise<T> {
  if (res.status === 401) {
    window.dispatchEvent(new CustomEvent("storeit:unauthenticated"));
    throw new ApiClientError("unauthenticated", "Not authenticated", 401);
  }

  if (!res.ok) {
    let code = "unknown";
    let message = `HTTP ${res.status}`;
    try {
      const body: ErrorResponse = await res.json();
      code = body.error.code;
      message = body.error.message;
    } catch {
      // use defaults
    }
    throw new ApiClientError(code, message, res.status);
  }

  if (res.status === 204) return undefined as T;
  return res.json();
}

const headers = { "Content-Type": "application/json" };
const opts: RequestInit = { credentials: "same-origin" };

export async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`, { ...opts });
  return handleResponse<T>(res);
}

export async function post<T>(path: string, body?: unknown): Promise<T> {
  const url = `${BASE}${path}`;
  const bodyStr = body !== undefined ? JSON.stringify(body) : undefined;
  try {
    const res = await fetch(url, {
      ...opts,
      method: "POST",
      headers,
      body: bodyStr,
    });
    return handleResponse<T>(res);
  } catch (err) {
    if (!navigator.onLine && bodyStr) {
      queueMutation({ url, method: "POST", headers, body: bodyStr });
      throw new ApiClientError("offline", "Queued for sync when online", 0);
    }
    throw err;
  }
}

export async function put<T>(path: string, body: unknown): Promise<T> {
  const url = `${BASE}${path}`;
  const bodyStr = JSON.stringify(body);
  try {
    const res = await fetch(url, {
      ...opts,
      method: "PUT",
      headers,
      body: bodyStr,
    });
    return handleResponse<T>(res);
  } catch (err) {
    if (!navigator.onLine) {
      queueMutation({ url, method: "PUT", headers, body: bodyStr });
      throw new ApiClientError("offline", "Queued for sync when online", 0);
    }
    throw err;
  }
}

export async function del(path: string): Promise<void> {
  const url = `${BASE}${path}`;
  try {
    const res = await fetch(url, {
      ...opts,
      method: "DELETE",
    });
    return handleResponse<void>(res);
  } catch (err) {
    if (!navigator.onLine) {
      queueMutation({ url, method: "DELETE", headers: {} });
      throw new ApiClientError("offline", "Queued for sync when online", 0);
    }
    throw err;
  }
}

export async function postMultipart<T>(
  path: string,
  formData: FormData,
): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...opts,
    method: "POST",
    body: formData,
  });
  return handleResponse<T>(res);
}

export function fileUrl(path: string): string {
  return `${BASE}${path}`;
}
