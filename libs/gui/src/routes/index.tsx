import { useTask$, useSignal, component$ } from '@builder.io/qwik';
import { useTaskContext } from '~/provider/task-context';
import { CreateTaskInput } from '~/ui/logics/dashboard/create-task-input';
import { OngoingTask } from '~/ui/logics/dashboard/ongoing-task';
import { TaskList } from '~/ui/logics/dashboard/task-list';
import { cn } from '~/utils/cn';
import type { DocumentHead } from '@builder.io/qwik-city';
import type { Task } from '~/api';

type DashboardProps = {
  class: string;
};

export default component$((props: DashboardProps) => {
  const taskContext = useTaskContext();
  const sortedTasks = useSignal<Task[]>([]);

  useTask$(({ track }) => {
    track(() => JSON.stringify(taskContext.tasks));
    const taskList = Object.values(taskContext.tasks).sort((a, b) => b.start - a.start);
    sortedTasks.value = taskList;
    return () => {};
  });

  return (
    <div class={cn('w-full mb-4', props.class)}>
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

export const head: DocumentHead = { title: 'home' };
