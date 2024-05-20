import { useComputed$, component$, $ } from '@builder.io/qwik';
import { getProjectMetadata } from '~/store/projects-store';
import { ProjectCard, type MenuCardData } from './project-card';
import type { PropFunction } from '@builder.io/qwik';
import type { ProjectMetadata } from '~/store/projects-store';

type MenuContentProps = {
  value: string;
  onSubmit$: PropFunction<() => void>;
};

export const toMenuCardData = async (s: string): Promise<MenuCardData | undefined> => {
  const tokens = s.trim().split(' ');

  if (s.trim() === '') return undefined;

  const tags: string[] = [];
  let project: string | undefined = undefined;

  const task_name = tokens
    .filter((t) => {
      const trimmed = t.trim();
      const startChar = trimmed[0];

      if (startChar === '@') {
        if (trimmed.length > 1) {
          project = trimmed.slice(1);
        }
        return false;
      } else if (startChar === '#') {
        if (trimmed.length > 1) {
          tags.push(trimmed.slice(1));
        }
        return false;
      }
      return true;
    })
    .join(' ');

  let projectMetadata: ProjectMetadata | undefined = undefined;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
  if (project) {
    projectMetadata = await getProjectMetadata(project);
  }
  return { tags, project, task_name, projectMetadata } as any;
};

export const MenuContent = component$((props: MenuContentProps) => {
  const data = useComputed$(() => toMenuCardData(props.value));

  return (
    <div>
      {data.value && (
        <>
          <div class="mx-2 my-2">
            <ProjectCard onClick$={$(() => props.onSubmit$())} selected={true} data={data.value} />
          </div>
        </>
      )}
      {!data.value && (
        <p class="flex items-center justify-center h-12 text-sm text-space-500">Start typing...</p>
      )}
    </div>
  );
});
