import type { Task, SyncRequest, SyncResponse, AuthResponse } from "./types";

// the backend lives here. hardcoded for now, env var later.
let API_BASE = "https://go2do-server.ideneyesa.workers.dev";

// swap to local dev server if running locally
if (typeof window !== "undefined" && window.location.hostname === "localhost") {
  API_BASE = "http://localhost:8787";
}

// unique device id for this browser session
function getDeviceId(): string {
  if (typeof window === "undefined") return "server";
  let id = localStorage.getItem("go2do_device_id");
  if (!id) {
    id = `web-${crypto.randomUUID()}`;
    localStorage.setItem("go2do_device_id", id);
  }
  return id;
}

export { getDeviceId };

// --- auth ---

export async function login(email: string, password: string): Promise<AuthResponse> {
  let res = await fetch(`${API_BASE}/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },

    body: JSON.stringify({ email, password }),
  });

  if (!res.ok) {
    let data = await res.json().catch(() => ({ error: "login failed" }));
    throw new Error((data as any).error || "login failed");
  }

  let data: AuthResponse = await res.json();

  // stash the token and user_id for sync requests
  localStorage.setItem("go2do_token", data.token);
  localStorage.setItem("go2do_user_id", data.user_id);

  return data;
}

export async function register(email: string, password: string): Promise<AuthResponse> {
  let res = await fetch(`${API_BASE}/auth/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },

    body: JSON.stringify({ email, password }),
  });

  if (!res.ok) {
    let data = await res.json().catch(() => ({ error: "registration failed" }));
    throw new Error((data as any).error || "registration failed");
  }

  return await res.json();
}

export function logout() {
  localStorage.removeItem("go2do_token");
  localStorage.removeItem("go2do_user_id");
  localStorage.removeItem("go2do_last_sync");
  window.location.href = "/login";
}

export function getToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem("go2do_token");
}

export function getUserId(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem("go2do_user_id");
}

export function isAuthenticated(): boolean {
  return !!getToken();
}

// --- sync ---

export async function syncTasks(changes: Task[]): Promise<SyncResponse> {
  let token = getToken();
  if (!token) throw new Error("not authenticated");

  let lastSync = parseInt(localStorage.getItem("go2do_last_sync") || "0", 10);

  let res = await fetch(`${API_BASE}/sync`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },

    body: JSON.stringify({ last_sync: lastSync, changes } satisfies SyncRequest),
  });

  if (res.status === 401) {
    logout();
    throw new Error("session expired");
  }

  if (!res.ok) {
    let data = await res.json().catch(() => ({ error: "sync failed" }));
    throw new Error((data as any).error || "sync failed");
  }

  let data: SyncResponse = await res.json();

  // save server time so next sync is incremental
  localStorage.setItem("go2do_last_sync", data.server_time.toString());

  return data;
}
