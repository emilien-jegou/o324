import { $ } from '@builder.io/qwik';
import { listen } from '@tauri-apps/api/event';
import { safeifyAsync } from '~/utils/result';
import type { Task } from './definitions';
import type { EventCallback } from '@tauri-apps/api/event';
import type { XOR } from 'ts-essentials';

type TaskActionPayload = XOR<{ Upsert: Task }, { Delete: string }>;

const safeListen = safeifyAsync(listen);

export const taskActionListener = $(async (listener: EventCallback<TaskActionPayload>) => {
  if (typeof window === 'undefined') {
    console.error('invalid call from server');
  } else {
    await safeListen('task_action', listener);
  }
});

export const configReloadListener = $(async (listener: EventCallback<void>) => {
  if (typeof window === 'undefined') {
    console.error('invalid call from server');
  } else {
    await safeListen('config_reload', listener);
  }
});
