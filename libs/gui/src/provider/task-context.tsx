import {
  $,
  useStore,
  Slot,
  createContextId,
  useComputed$,
  useContext,
  useContextProvider,
  component$,
  useTask$,
} from '@builder.io/qwik';

import { taskActionListener, configReloadListener, listLastTasks, type Task } from '~/api';
import { useTrayIconUpdater } from '~/hooks/use-tray-icon-updater';
import type { Signal } from '@builder.io/qwik';

export type TaskContextData = {
  tasks: Record<string, Task>;
  currentTask: Signal<Task | undefined>;
};

const taskContext = createContextId<TaskContextData>('TaskContext');

export const useTaskContext = () => useContext(taskContext);

export const TaskContextProvider = component$(() => {
  const tasks = useStore<Record<string, Task>>({});

  const refresh$ = $(async () => {
    const result = await listLastTasks(30);
    Object.keys(tasks).forEach((key) => delete tasks[key]);
    result.sort((a, b) => b.start - a.start);
    result.forEach((t) => (tasks[t.ulid] = t));
  });

  useTask$(async () => {
    await taskActionListener((e) => {
      if (e.payload.Upsert) {
        const newTask = e.payload.Upsert;
        const currentTask = tasks[newTask.ulid] ?? {};
        tasks[newTask.ulid] = { ...currentTask, ...newTask };
      } else if (e.payload.Delete) {
        const taskId = e.payload.Delete;
        delete tasks[taskId];
      }
    });

    await configReloadListener(refresh$);
    await refresh$();
  });

  const currentTask = useComputed$(() => Object.values(tasks).find((t) => !t.end));

  useContextProvider(taskContext, { tasks, currentTask });

  useTrayIconUpdater(currentTask);

  return <Slot />;
});
