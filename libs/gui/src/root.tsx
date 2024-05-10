import { $, useVisibleTask$, useTask$, useSignal, useStore, component$ } from "@builder.io/qwik";
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event'

import "./global.css";

type Task = {
  ulid: string,
  end: number | null,
  project: string | null,
  start: number,
  tags: string[],
  task_name: string,
  __hash: number
}

export default component$(() => {
  const tasks = useStore<Record<string, Task>>({});
  const sortedTasks = useSignal<Task[]>([]);
  const startTaskModalIsVisible = useSignal<boolean>(false);
  const editModalTaskId = useSignal<Task['ulid'] | null>(null);
  const clock = useSignal<number | null>(null);

  const refresh$ = $(async () => {
    let result: Task[] = await invoke('list_last_tasks', { count: 30 })
    Object.keys(tasks).forEach(key => delete tasks[key]);
    result.sort((a, b) => b.start - a.start);
    result.forEach((t) => tasks[t.ulid] = t);
  });

  const stopCurrentTask$ = $(async () => {
    await invoke('stop_current_task', {});
  });

  const deleteTaskById$ = $(async (task_id: Task['ulid']) => {
    await invoke('delete_task_by_ulid', { ulid: task_id });
  });

  const synchronize$ = $(async () => {
    await invoke('synchronize_tasks', {});
  });

  const getTimer$ = $(async (time: number) => {
    const secs = String(Math.floor(time % 60)).padStart(2, '0');
    const minutes = String(Math.floor((time % 3600) / 60)).padStart(2, '0');
    const hours = String(Math.floor(time / 3600)).padStart(2, '0');

    return `${hours}:${minutes}:${secs}`;

  });

  useVisibleTask$(async () => {
    await listen('task-action', (e: any) => {
      console.log(e);
      if (e.payload.Upsert) {
        const newTask = e.payload.Upsert;
        const currentTask = tasks[newTask.ulid] ?? {};
        tasks[newTask.ulid] = { ...currentTask, ...newTask };
      } else if (e.payload.Delete) {
        const taskId = e.payload.Delete;
        delete tasks[taskId];
      }
    });
    await refresh$();
  });

  useTask$(({ track }) => {
    track(() => JSON.stringify(tasks));
    let taskList = Object.values(tasks).sort((a, b) => b.start - a.start);
    sortedTasks.value = taskList;

    let currentTask = taskList.find((t) => !t.end);

    if (currentTask) {
      // Create clock
      const computeClockValue = () => {
        clock.value = (Date.now() - currentTask.start) / 1000;
      }

      computeClockValue();
      let intervalId = setInterval(computeClockValue, 1000);

      return () => {
        clock.value = null;
        clearTimeout(intervalId);
      }
    }
    console.info(sortedTasks.value);
  })

  return (
    <html>
      <head>
        <meta charSet="utf-8" />
        <link rel="manifest" href="/manifest.json" />
      </head>
      <body lang="en">
        <div>
          <div class="flex items-center justify-between m-4">
            <div class="flex gap-4">
              <button class="border cursor-pointer hover:bg-slate-200 bg-slate-100 px-4 py-2"
                onClick$={refresh$}>Refresh</button>
              <button class="border cursor-pointer hover:bg-slate-200 bg-slate-100 px-4 py-2"
                onClick$={() => {
                  startTaskModalIsVisible.value = true;
                }}>Start new task</button>
              <button class="border cursor-pointer hover:bg-slate-200 bg-slate-100 px-4 py-2"
                onClick$={stopCurrentTask$}>Stop current task</button>
              <button class="border cursor-pointer hover:bg-slate-200 bg-slate-100 px-4 py-2"
                onClick$={synchronize$}>Synchronize</button>
            </div>
            <p class="font-bold text-xl">{clock.value !== null ? getTimer$(clock.value) : ''}</p>
          </div>
          <table class="text-sm w-full">
            <thead class="bg-blue-800 text-white text-left">
              <tr>
                <th class="px-2 py-1">ID</th>
                <th class="px-2">Name</th>
                <th class="px-2">Project</th>
                <th class="px-2">Tags</th>
                <th class="px-2">Start - End</th>
                <th class="px-2"></th>
              </tr>
            </thead>
            <tbody>
              {sortedTasks.value.map((task) =>
                <tr key={[task.ulid, task.__hash].join()} class="even:bg-blue-50 text-left">
                  <th class="px-2 py-1">{task.ulid}</th>
                  <td class="px-2">{task.task_name}</td>
                  <td class="px-2">{task.project}</td>
                  <td class="px-2">{task.tags.join(', ')}</td>
                  <td class="px-2">{task.start} - {task.end ?? 'Nil'}</td>
                  <td class="flex gap-2 px-2">
                    <button onClick$={() => { editModalTaskId.value = task.ulid }} class="border cursor-pointer hover:bg-slate-200 bg-slate-100 px-1 py-1">edit</button>
                    <button onClick$={() => deleteTaskById$(task.ulid)} class="border cursor-pointer hover:bg-slate-200 bg-slate-100 px-1 py-1">X</button>
                  </td>

                </tr>
              )}
            </tbody>
          </table>
        </div>

        {editModalTaskId.value !== null && <div onClick$={() => {
          editModalTaskId.value = null;
        }} class="absolute top-0 left-0 z-50 w-screen h-screen bg-black/10">
          <div onClick$={(e) => e.stopPropagation()} class="w-md p-6 mx-auto mt-[10vh] max-h-[80vh] overflow-y-auto bg-white border border-black shadow-sm">
            <h1 class="text-xl font-bold mb-6">Start new task</h1>
            <form preventdefault:submit onSubmit$={async (e: any) => {
              const formData = new FormData(e.target);

              const task_name = formData.get('task_name') || undefined;
              const project = formData.get('project') || undefined;
              const tags = formData.get('tags') ? String(formData.get('tags')).split(',') : undefined;

              console.info('invoking', { task_name, project, tags });
              const t = await invoke('edit_task', { ulid: editModalTaskId.value, data: { task_name, project, tags } });

              console.info(t);
              console.info('there');
              editModalTaskId.value = null;
            }} class="flex flex-col gap-4">
              <div class="flex gap-2 flex-row">
                <label for="task_name" class="font-medium w-[150px]">Task name</label>
                <input id="task_name" class="h-8 w-full" name="task_name" />
              </div>

              <div class="flex gap-2 flex-row">
                <label for="project" class="font-medium w-[150px]">Project</label>
                <input id="project" class="h-8 w-full" name="project" />
              </div>

              <div class="flex gap-2 flex-row">
                <label for="tags" class="font-medium w-[150px]">Tags</label>
                <input id="tags" class="h-8 w-full" name="tags" />
              </div>

              <button type="submit" class="border cursor-pointer hover:bg-slate-200 bg-slate-100 px-4 py-2"
              >Submit</button>
            </form>
          </div>
        </div>}

        {startTaskModalIsVisible.value == true && <div onClick$={() => {
          startTaskModalIsVisible.value = false;
        }} class="absolute top-0 left-0 z-50 w-screen h-screen bg-black/10">
          <div onClick$={(e) => e.stopPropagation()} class="w-md p-6 mx-auto mt-[10vh] max-h-[80vh] overflow-y-auto bg-white border border-black shadow-sm">
            <h1 class="text-xl font-bold mb-6">Start new task</h1>
            <form preventdefault:submit onSubmit$={async (e: any) => {
              const formData = new FormData(e.target);

              const task_name = String(formData.get('task_name') ?? '');
              const project = String(formData.get('project') ?? '');
              const tags = String(formData.get('tags') || '').split(',');

              console.info('invoking');
              await invoke('start_new_task', { data: { task_name, project, tags } });
              console.info('there');
              startTaskModalIsVisible.value = false;
            }} class="flex flex-col gap-4">
              <div class="flex gap-2 flex-row">
                <label for="task_name" class="font-medium w-[150px]">Task name *</label>
                <input id="task_name" class="h-8 w-full" name="task_name" />
              </div>

              <div class="flex gap-2 flex-row">
                <label for="project" class="font-medium w-[150px]">Project</label>
                <input id="project" class="h-8 w-full" name="project" />
              </div>

              <div class="flex gap-2 flex-row">
                <label for="tags" class="font-medium w-[150px]">Tags</label>
                <input id="tags" class="h-8 w-full" name="tags" />
              </div>

              <button type="submit" class="border cursor-pointer hover:bg-slate-200 bg-slate-100 px-4 py-2"
              >Submit</button>
            </form>
          </div>
        </div>}
      </body>
    </html>
  );
});
