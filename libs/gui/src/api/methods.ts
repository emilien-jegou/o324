import { invoke } from '@tauri-apps/api/core';
import { Task } from './definitions';

export const listLastTasks = (count: number): Promise<Task[]> => invoke('list_last_tasks', { count });

export const stopCurrentTask = () => invoke('stop_current_task', {});

export const deleteTaskByUlid = (task_id: string) => invoke('delete_task_by_ulid', { ulid: task_id });

export const synchronizeTasks = () => invoke('synchronize_tasks', {});

export const getCurrentConfig = () => invoke('get_current_config', {});

// TODO: can we set remove the project value using this method?
type EditTaskUpdateInput = { task_name: string | null, project: string | null, tags: string[] | null };
export const editTask = (task_id: string, task_update: EditTaskUpdateInput) =>
  invoke('edit_task', { ulid: task_id, data: task_update });

type StartNewTaskInput = { task_name: string, project: string | null, tags: string[] };
export const startNewTask = (task: StartNewTaskInput) => invoke('start_new_task', { data: task });

type SaveNewConfigInput = Record<string, any>;
export const saveNewConfig = (config: SaveNewConfigInput) => invoke('save_new_config', { config: config });

export const loadProfile = (profile: string) => invoke('load_profile', { profile });
