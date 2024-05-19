import { listen } from '@tauri-apps/api/event';
import type { Task } from './definitions';
import type { XOR } from '../utils/xor';
import type { EventCallback } from '@tauri-apps/api/event';

type TaskActionPayload = XOR<{ Upsert: Task }, { Delete: string }>;

export const taskActionListener = (listener: EventCallback<TaskActionPayload>) =>
  listen('task_action', listener);

export const configReloadListener = (listener: EventCallback<void>) =>
  listen('config_reload', listener);
