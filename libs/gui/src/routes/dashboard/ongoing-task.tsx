import { $, useComputed$, useTask$, useSignal, component$ } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { stopCurrentTask, type Task } from '~/api';
import { getOrCreateProjectMetadata } from '~/store/projects-store';
import { ProjectLabel } from '~/ui/common/project-label/project-label';
import { PauseIcon } from '~/ui/icons/pause';
import { StopIcon } from '~/ui/icons/stop';
import type { JSXChildren, PropFunction } from '@builder.io/qwik';
//import { useTaskContext } from '~/provider/task-context';

type OngoingTaskButtonProps = {
  onClick$?: PropFunction<() => void>;
  onMouseEnter$?: PropFunction<() => void>;
  onMouseLeave$?: PropFunction<() => void>;
  icon: JSXChildren;
  class?: string;
};

export const OngoingTaskButton = (props: OngoingTaskButtonProps) => (
  <button
    class={twMerge('text-space-100 p-2 hover:text-accent-300', props.class)}
    onClick$={props.onClick$}
    onMouseEnter$={props.onMouseEnter$}
    onMouseLeave$={props.onMouseLeave$}
  >
    <div>{props.icon}</div>
  </button>
);

type OngoingTaskProps = {
  class: string;
  task: Task;
};

export const getTimer = async (time: number) => {
  const secs = String(Math.floor(time % 60)).padStart(2, '0');
  const minutes = String(Math.floor((time % 3600) / 60)).padStart(2, '0');
  const hours = String(Math.floor(time / 3600)).padStart(2, '0');

  return `${hours}:${minutes}:${secs}`;
};

export const OngoingTask = component$((props: OngoingTaskProps) => {
  // TODO: this should be done through the app reload event
  const clock = useSignal<number | null>(null);

  useTask$(() => {
    const computeClockValue = () => {
      clock.value = (Date.now() - props.task.start) / 1000;
    };

    computeClockValue();
    const intervalId = setInterval(computeClockValue, 1000);

    return () => {
      clock.value = null;
      clearTimeout(intervalId);
    };
  });

  const project = props.task.project;
  const metadata = useComputed$(() => {
    if (!project) return undefined;
    return getOrCreateProjectMetadata(project);
  });

  return (
    <div
      class={twMerge(
        'max-w-[100%] flex gap-4 items-center justify-between p-4 rounded-sm bg-space-700',
        props.class,
      )}
    >
      <div class="flex flex-0 shrink items-center gap-1 max-w-[80%]">
        <p class="font-bold text-xl mr-4">{clock.value !== null ? getTimer(clock.value) : ''}</p>

        <h2 class="text-lg font-medium one-line mr-2">{props.task.task_name}</h2>
        <div class="text-md text-space-400 font-semibold flex gap-2 items-center">
          {props.task.tags.map((x, idx) => (
            <p key={idx}>#{x}</p>
          ))}
        </div>

        <h2 class="text-xl font-medium one-line"></h2>
        {props.task.project && metadata.value && (
          <ProjectLabel
            size="md"
            color={metadata.value.projectColor}
            projectName={props.task.project}
            isNew={false}
          />
        )}
      </div>
      <div class="flex gap-1 pr-4">
        <button
          class="hover:underline"
          onClick$={$(async () => {
            await stopCurrentTask();
          })}
        >
          stop task
        </button>
      </div>
    </div>
  );
});
