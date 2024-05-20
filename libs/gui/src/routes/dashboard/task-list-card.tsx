import { $, useSignal, useComputed$, component$ } from '@builder.io/qwik';
import { format, intervalToDuration } from 'date-fns';
import { twMerge } from 'tailwind-merge';
import { deleteTaskByUlid, startNewTask, type Task } from '~/api';
import { usePortalContext } from '~/provider/portal-context';
import { AlertDialog } from '~/ui/common/alert-dialog';
import { ProjectLabel } from '~/ui/common/project-label/project-label';
import { EditTaskModal } from './edit-task-modal';
import type { PropFunction } from '@builder.io/qwik';
import type { ProjectMetadata } from '~/store/projects-store';

type TaskListCardProps = {
  task: Task & { metadata?: ProjectMetadata };
};

export function readableFormatTime(secs: number) {
  const duration = intervalToDuration({ start: 0, end: secs * 1000 });

  if (duration.years) return `${duration.years} year${duration.years > 1 ? 's' : ''}`;
  else if (duration.months) return `${duration.months * 30 + (duration.weeks ?? 0) * 7} days`;
  else if (duration.weeks) return `${duration.weeks * 7} days`;
  else if (duration.days) return `${duration.days}d ${duration.hours ?? 0}h`;
  else if (duration.hours) return `${duration.hours}h ${duration.minutes}m`;
  else if (duration.minutes)
    return `${String(duration.minutes)} min${duration.minutes > 1 ? 's' : ''}`;
  else if (duration.seconds) {
    return `${String(duration.seconds)} sec${duration.seconds > 1 ? 's' : ''}`;
  }

  return '0 sec';
}

const formatAsHours = (time: number) => format(new Date(time), 'h:mmaaa');

export const TaskListCard = component$((props: TaskListCardProps) => {
  const containerIsHovered = useSignal(false);
  const playButtonIsHovered = useSignal(false);
  const deleteModalVisible = useSignal(false);
  const editModalVisible = useSignal(false);

  const formattedTime = useComputed$(() => {
    const start = props.task.start;
    const end = props.task.end ?? Date.now();
    const secs = Math.floor((end - start) / 1000);

    return readableFormatTime(secs);
  });

  return (
    <>
      <div
        onMouseEnter$={() => {
          containerIsHovered.value = true;
        }}
        onMouseLeave$={() => {
          containerIsHovered.value = false;
        }}
        class={twMerge(
          'flex justify-between gap-10 h-10 items-center border-t first:border-0 p-4 py-1 border-space-700',
          containerIsHovered.value && !playButtonIsHovered.value && 'bg-space-800',
        )}
      >
        <div class="flex gap-4 one-line items-center text-ellipsis overflow-hidden">
          <p class="font-semibold w-16 shrink-0 mr-4">{formattedTime}</p>
          <p class="one-line">
            <span>{props.task.task_name}</span>
            {props.task.tags.length !== 0 && (
              <span class="ml-4 text-space-400 one-line">
                {props.task.tags.map((t) => `#${t}`).join(' ')}
              </span>
            )}
          </p>
          {props.task.project && props.task.metadata && (
            <ProjectLabel
              size="sm"
              isNew={!!props.task.metadata.isNew}
              color={props.task.metadata.projectColor}
              projectName={props.task.project}
            />
          )}
        </div>
        <div
          class={twMerge(
            'hidden gap-4 items-center transition-opacity duration-20',
            containerIsHovered.value && 'flex',
          )}
        >
          <TextButton
            label="start task"
            onClick$={$(() => {
              startNewTask({
                task_name: props.task.task_name,
                project: props.task.project,
                tags: props.task.tags,
              });
            })}
          />
          <TextButton
            label="edit"
            onClick$={$(() => {
              editModalVisible.value = true;
            })}
          />
          <TextButton
            label="delete"
            onClick$={$(() => {
              deleteModalVisible.value = true;
            })}
          />
        </div>
        <EditTaskModal task={props.task} bind:show={editModalVisible} />
        <AlertDialog
          title={`Delete task "${props.task.task_name}"?`}
          description={`This action cannot be undone, are you sure you want to delete the task started at ${formatAsHours(props.task.start)} and stopped at ${formatAsHours(props.task.end!)}.`}
          bind:show={deleteModalVisible}
          onContinue$={$(() => {
            deleteTaskByUlid(props.task.ulid);
            deleteModalVisible.value = false;
          })}
        />
      </div>
    </>
  );
});

type TextButtonProps = {
  label: string;
  onClick$?: PropFunction<() => void>;
};

const TextButton = (props: TextButtonProps) => (
  <button
    onClick$={props.onClick$}
    class="text-space-300 hover:text-white hover:underline text-nowrap"
  >
    {props.label}
  </button>
);
