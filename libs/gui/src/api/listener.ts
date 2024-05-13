import { EventCallback, listen } from '@tauri-apps/api/event'
import { XOR } from '../utils/xor';
import { Task } from './definitions';

type TaskActionPayload =
  XOR<{ Upsert: Task }, { Delete: string }>;

export const taskActionListener = (listener: EventCallback<TaskActionPayload>) => listen('task_action', listener);

export const configReloadListener = (listener: EventCallback<void>) => listen('config_reload', listener);

