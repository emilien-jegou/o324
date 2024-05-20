import { twMerge } from 'tailwind-merge';
import { KeyboardHint } from '~/ui/common/keyboard-hint';
import { ProjectLabel } from '~/ui/common/project-label/project-label';
import { EnterKeyIcon } from '~/ui/icons/enter-key';
import { TimerIcon } from '~/ui/icons/timer-icon';
import type { PropFunction } from '@builder.io/qwik';
import type { ProjectMetadata } from '~/store/projects-store';

export type MenuCardData = {
  task_name: string;
  tags: string[];
} & (
  | { project?: undefined; projectMetadata?: undefined }
  | { project: string; projectMetadata: ProjectMetadata }
);

type ProjectCardProps = {
  data: MenuCardData;
  selected?: boolean;
  onClick$: PropFunction<() => void>;
};

export const ProjectCard = (props: ProjectCardProps) => {
  return (
    <button
      type="button"
      onClick$={() => props.onClick$()}
      class={twMerge(
        'w-full relative h-12 rounded-md shadow-sm flex items-center flec-col gap-2 px-3 pr-16 py-3 border-b text-sm border-space-600',
        props.selected && 'bg-space-600',
      )}
    >
      <TimerIcon />
      <p class="text-ellipsis overflow-hidden text-nowrap">{props.data.task_name}</p>
      <div class="flex gap-2 items-center">
        {props.data.tags.map((tag: string) => (
          <p key={tag} class="text-space-400 font-semibold">
            #{tag}
          </p>
        ))}
      </div>
      {props.data.project && (
        <ProjectLabel
          projectName={props.data.project}
          color={props.data.projectMetadata.projectColor}
          isNew={!!props.data.projectMetadata.isNew}
        />
      )}

      <KeyboardHint
        class={twMerge(
          'text-white absolute right-4 top-1/2 -translate-y-1/2 transition-opacity',
          !props.selected && 'opacity-0',
        )}
        hint={<EnterKeyIcon />}
      />
    </button>
  );
};
