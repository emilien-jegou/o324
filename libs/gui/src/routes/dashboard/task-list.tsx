import { useComputed$, component$ } from '@builder.io/qwik';
import { format, isToday, isYesterday, startOfDay } from 'date-fns';
import { twMerge } from 'tailwind-merge';
import { useTaskContext } from '~/provider/task-context';
import { getOrCreateProjectMetadata, type ProjectMetadata } from '~/store/projects-store';
import { TaskListCard, readableFormatTime } from './task-list-card';
import type { Task } from '~/api';

type TaskListProps = {
  class?: string;
};

function formatDateRelativeToNow(timestamp: number) {
  if (isToday(timestamp)) {
    return 'Today';
  } else if (isYesterday(timestamp)) {
    return 'Yesterday';
  } else {
    return format(timestamp, 'EEEE d MMM');
  }
}

function getMidnightTimestamp(timestamp: number): number {
  const midnightDate: Date = startOfDay(new Date(timestamp));
  return midnightDate.getTime();
}

export const TaskList = component$((props: TaskListProps) => {
  const taskContext = useTaskContext();

  const tasks = useComputed$(async () => {
    const taskRecord: Record<
      string,
      {
        label: string;
        timeMs: number;
        list: Record<string, Task & { metadata?: ProjectMetadata }>;
      }
    > = {};
    const entries = Object.entries(taskContext.tasks).filter(([_, { end }]) => !!end);

    for (const [k, v] of entries) {
      const d = getMidnightTimestamp(v.end!);
      let record = taskRecord[d];

      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
      if (!record) {
        record = { label: formatDateRelativeToNow(d), timeMs: 0, list: {} };
        taskRecord[d] = record;
      }

      const metadata = v.project ? await getOrCreateProjectMetadata(v.project) : undefined;

      record.list[k] = { ...v, metadata };
      if (v.end) {
        record.timeMs = record.timeMs + (v.end - v.start);
      }
    }

    return taskRecord;
  });

  return (
    <div class={twMerge('overflow-auto h-full', props.class)}>
      <div>
        {Object.entries(tasks.value)
          .sort((a, b) => b[0].localeCompare(a[0]))
          .map(([k, v], idx) => (
            <div key={k + idx}>
              <p class="bg-space-1000 flex items-center justify-between px-4 mt-1 text-sm font-medium py-1">
                <span>{v.label}</span>
                <span>{readableFormatTime(Math.floor(v.timeMs / 1000))}</span>
              </p>
              <div>
                {Object.entries(v.list)
                  .sort((a, b) => b[0].localeCompare(a[0]))
                  .map(([k, v], idx) => (
                    <TaskListCard key={k + idx} task={v} />
                  ))}
              </div>
            </div>
          ))}
      </div>
    </div>
  );
});
