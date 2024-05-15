import { $, useVisibleTask$, useComputed$, useTask$, Resource, useResource$, useSignal, useStore, component$ } from "@builder.io/qwik";
import { Task, configReloadListener, deleteTaskByUlid, editTask, getCurrentConfig, listLastTasks, loadProfile, saveNewConfig, startNewTask, stopCurrentTask, synchronizeTasks, taskActionListener, updateTrayIcon } from "../api";

type DashboardProps = {
  class: string;
}

export const Dashboard = component$((props: DashboardProps) => {
  const tasks = useStore<Record<string, Task>>({});
  const sortedTasks = useSignal<Task[]>([]);
  const startTaskModalIsVisible = useSignal<boolean>(false);
  const configModalIsVisible = useSignal<boolean>(false);
  const chooseProfileModalIsVisible = useSignal<boolean>(false);
  // TODO: this should be done through the app reload event
  const editModalTaskId = useSignal<Task['ulid'] | null>(null);
  const clock = useSignal<number | null>(null);

  const refresh$ = $(async () => {
    let result = await listLastTasks(30);
    Object.keys(tasks).forEach(key => delete tasks[key]);
    result.sort((a, b) => b.start - a.start);
    result.forEach((t) => tasks[t.ulid] = t);
  });

  const configResource = useResource$(({ track }) => {
    track(() => configModalIsVisible.value);
    if (configModalIsVisible.value == false) {
      return null;
    } else {
      return getCurrentConfig();
    }
  });

  const getTimer$ = $(async (time: number) => {
    const secs = String(Math.floor(time % 60)).padStart(2, '0');
    const minutes = String(Math.floor((time % 3600) / 60)).padStart(2, '0');
    const hours = String(Math.floor(time / 3600)).padStart(2, '0');

    return `${hours}:${minutes}:${secs}`;

  });

  useVisibleTask$(async () => {
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

  let currentTask = useComputed$(() => Object.values(tasks).find((t) => !t.end));

  useTask$(({ track }) => {
    track(() => JSON.stringify(tasks));
    let taskList = Object.values(tasks).sort((a, b) => b.start - a.start);
    sortedTasks.value = taskList;

    const current = currentTask.value

    if (current) {
      // Create clock
      const computeClockValue = () => {
        clock.value = (Date.now() - current.start) / 1000;
      }

      computeClockValue();
      let intervalId = setInterval(computeClockValue, 1000);

      return () => {
        clock.value = null;
        clearTimeout(intervalId);
      }
    }

    return () => { };
  })

  useTask$(({ track }) => {
    track(() => !currentTask.value);
    updateTrayIcon(!!currentTask.value);
  });

  return (
    <div class={props.class}>
      <div>
        <div class="flex items-center justify-between m-4">
          <div class="flex gap-4">
            <button class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              onClick$={refresh$}>Refresh</button>
            <button class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              onClick$={() => {
                startTaskModalIsVisible.value = true;
              }}>Start new task</button>
            <button class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              onClick$={() => stopCurrentTask()}>Stop current task</button>
            <button class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              onClick$={() => synchronizeTasks()}>Synchronize</button>
            <button class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              onClick$={() => {
                configModalIsVisible.value = true;
              }}>config</button>
            <button class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              onClick$={() => {
                chooseProfileModalIsVisible.value = true;
              }}>profile</button>
          </div>
          <p class="font-bold text-xl">{clock.value !== null ? getTimer$(clock.value) : ''}</p>
        </div>
        <div class="w-full px-4">
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
                    <button onClick$={() => { editModalTaskId.value = task.ulid }} class="border border-gray-300 cursor-pointer hover:bg-slate-200 bg-slate-100 px-1 py-1">edit</button>
                    <button onClick$={() => deleteTaskByUlid(task.ulid)} class="border  border-gray-300 cursor-pointer hover:bg-slate-200 bg-slate-100 px-1 py-1">X</button>
                  </td>

                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {
        editModalTaskId.value !== null && <div onClick$={() => {
          editModalTaskId.value = null;
        }} class="absolute top-0 left-0 z-50 w-screen h-screen bg-black/10">
          <div onClick$={(e) => e.stopPropagation()} class="w-md p-6 mx-auto mt-[10vh] max-h-[80vh] overflow-y-auto bg-white border border-gray-300 shadow-sm">
            <h1 class="text-xl font-bold mb-6">Update a task</h1>
            <form preventdefault:submit onSubmit$={async (e: any) => {
              const formData = new FormData(e.target);

              const task_name = String(formData.get('task_name')) || null;
              const project = String(formData.get('project')) || null;
              const tags = formData.get('tags') ? String(formData.get('tags')).split(',') : null;

              await editTask(editModalTaskId.value!, { task_name, project, tags });

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

              <button type="submit" class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              >Submit</button>
            </form>
          </div>
        </div>
      }

      {
        startTaskModalIsVisible.value == true && <div onClick$={() => {
          startTaskModalIsVisible.value = false;
        }} class="absolute top-0 left-0 z-50 w-screen h-screen bg-black/10">
          <div onClick$={(e) => e.stopPropagation()} class="w-md p-6 mx-auto mt-[10vh] max-h-[80vh] overflow-y-auto bg-white border border-gray-300 shadow-sm">
            <h1 class="text-xl font-bold mb-6">Start new task</h1>
            <form preventdefault:submit onSubmit$={async (e: any) => {
              const formData = new FormData(e.target);

              const task_name = String(formData.get('task_name') ?? '');
              const project = String(formData.get('project') ?? '');
              const tags = String(formData.get('tags') || '').split(',');

              await startNewTask({ task_name, project, tags });
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

              <button type="submit" class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              >Submit</button>
            </form>
          </div>
        </div>
      }

      {
        configModalIsVisible.value == true && <div onClick$={() => {
          configModalIsVisible.value = false;
        }} class="absolute top-0 left-0 z-50 w-screen h-screen bg-black/10">
          <div onClick$={(e) => e.stopPropagation()} class="w-xl p-6 mx-auto mt-[10vh] max-h-[80vh] overflow-y-auto bg-white border border-gray-300 shadow-sm">
            <h1 class="text-xl font-bold mb-6">config</h1>
            <form preventdefault:submit onSubmit$={async (e: any) => {
              const formData = new FormData(e.target);

              const config = String(formData.get('config') ?? '');

              const result = saveNewConfig(JSON.parse(config));
              configModalIsVisible.value = false;
            }} class="flex flex-col gap-4">
              <Resource
                value={configResource}
                onPending={() => <></>}
                onRejected={(reason: any) => <div>Error: {reason}</div>}
                onResolved={(data: any) =>
                  <textarea id="config" class="h-80 w-full" name="config" defaultValue={JSON.stringify(data, null, 2)} />
                }
              />

              <button type="submit" class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              >Submit</button>
            </form>
          </div>
        </div>
      }

      {
        chooseProfileModalIsVisible.value == true && <div onClick$={() => {
          chooseProfileModalIsVisible.value = false;
        }} class="absolute top-0 left-0 z-50 w-screen h-screen bg-black/10">
          <div onClick$={(e) => e.stopPropagation()} class="w-xl p-6 mx-auto mt-[10vh] max-h-[80vh] overflow-y-auto bg-white border border-gray-300 shadow-sm">
            <h1 class="text-xl font-bold mb-6">config</h1>
            <form preventdefault:submit onSubmit$={async (e: any) => {
              const formData = new FormData(e.target);
              const profile = String(formData.get('profile_name'));
              await loadProfile(profile);
              chooseProfileModalIsVisible.value = false;
            }} class="flex flex-col gap-4">

              <div class="flex gap-2 flex-row">
                <label for="profile_name" class="font-medium w-[150px]">Profile name *</label>
                <input id="profile_name" class="h-8 w-full" name="profile_name" />
              </div>
              <button type="submit" class="border cursor-pointer border-gray-300 hover:bg-slate-200 bg-slate-100 px-4 py-2"
              >Submit</button>
            </form>
          </div>
        </div>
      }
    </div>
  );
});

