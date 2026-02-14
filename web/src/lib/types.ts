// task shape matching the server schema
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

export type SyncResponse = {
  updates: Task[];
  server_time: number;
};

export type AuthResponse = {
  message: string;
  token: string;
  user_id: string;
};
