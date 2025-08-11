import { $ } from '@builder.io/qwik';
import { invoke } from '@tauri-apps/api/core';
import { failure, success, type Result } from '~/utils/result';
import type { Task } from './definitions';
import type { QRL } from '@builder.io/qwik';

const method = function <Ret, T extends unknown[]>(
  name: string,
  m?: QRL<(...args: T) => Record<string, unknown>>,
): QRL<(...args: T) => Promise<Result<Ret, String>>> {
  if (typeof window !== 'undefined') {
    (window as any).__METHOD = { ...((window as any).__METHOD ?? {}), [name]: m };
  }
  return $(async (...args: T) => {
    const x = await (m?.(...args) ?? Promise.resolve({}));
    try {
      return success(await invoke(name, x));
    } catch (e) {
      if (String(e) === "TypeError: Cannot read properties of undefined (reading 'invoke')") {
        return failure(`running invoke method '${name}' outside of tauri`);
      } else {
        return failure(String(e));
      }
    }
  });
};

export const listLastTasks = method<Task[], [number]>(
  'list_last_tasks',
  $((count: number) => ({
    count,
  })),
);

export const stopCurrentTask = method('stop_current_task');

export const deleteTaskByUlid = method(
  'delete_task_by_ulid',
  $((task_id: string) => ({
    ulid: task_id,
  })),
);

export const synchronizeTasks = method('synchronize_tasks');

export const getCurrentConfig = method('get_current_config');

// TODO: can we set remove the project value using this method?
type EditTaskUpdateInput = {
  task_name: string | null;
  project: string | null;
  tags: string[] | null;
};

export const editTask = method(
  'edit_task',
  $((task_id: string, task_update: EditTaskUpdateInput) => ({ ulid: task_id, data: task_update })),
);

type StartNewTaskInput = { task_name: string; project: string | null; tags: string[] };
export const startNewTask = method(
  'start_new_task',
  $((task: StartNewTaskInput) => ({ data: task })),
);

type SaveNewConfigInput = Record<string, any>;
export const saveNewConfig = method(
  'save_new_config',
  $((config: SaveNewConfigInput) => ({
    config: config,
  })),
);

export const loadProfile = method(
  'load_profile',
  $((profile: string) => ({ profile })),
);

export const updateTrayIcon = method(
  'update_tray_icon',
  $((active: boolean) => ({ active })),
);
