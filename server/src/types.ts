export type Bindings = {
  DB: D1Database;
  GO2DO_KV: KVNamespace;
  JWT_SECRET: string;
};

export type Task = {
  id: string;
  user_id: string;
  created_at: number;
  updated_at: number;
  deleted_at: number | null;
  content: string;
  completed: boolean;
  pinned: boolean;
  version: number;
  device_id: string;
};

export type SyncRequest = {
  last_sync: number;
  changes: Task[];
};

export type User = {
  id: string;
  email: string;
  password_hash: string;
  created_at: number;
};
