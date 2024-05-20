import { useTask$, useSignal, component$ } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { useTaskContext } from '~/provider/task-context';
import { CreateTaskInput } from './create-task-input';
import { OngoingTask } from './ongoing-task';
import { TaskList } from './task-list';
import type { Task } from '~/api';

type DashboardProps = {
  class: string;
};

export const Dashboard = component$((props: DashboardProps) => {
  const taskContext = useTaskContext();
  const sortedTasks = useSignal<Task[]>([]);

  useTask$(({ track }) => {
    track(() => JSON.stringify(taskContext.tasks));
    const taskList = Object.values(taskContext.tasks).sort((a, b) => b.start - a.start);
    sortedTasks.value = taskList;
    return () => {};
  });

  return (
    <div class={twMerge('mb-4', props.class)}>
      <div class="h-[7.5rem] pt-6 w-full">
        {!taskContext.currentTask.value && <CreateTaskInput />}
        {!!taskContext.currentTask.value && (
          <OngoingTask class="mx-4 mt-2" task={taskContext.currentTask.value} />
        )}
      </div>
      <TaskList class="w-full px-4" />
    </div>
  );
});
