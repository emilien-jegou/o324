import { twMerge } from 'tailwind-merge';
import type { ProjectColor } from '~/store/projects-store';

type ProjectLabelProps = {
  class?: string;
  projectName: string;
  color: ProjectColor;
  isNew: boolean;
  size?: 'sm' | 'md';
};

export const ProjectLabel = ({ size = 'md', ...props }: ProjectLabelProps) => (
  <p
    class={twMerge(
      'text-black/90 border-2 text-sm  border-space-600 one-line shrink-0',
      size === 'sm' && 'rounded-md px-1 py-0 font-semibold ',
      size === 'md' && 'rounded-md p-1 px-2 font-semibold ',
      props.class,
    )}
    style={{ backgroundColor: props.color.dark as any }}
  >
    {props.isNew === true && 'New project: '}@{props.projectName}
  </p>
);
