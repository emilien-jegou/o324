import { twMerge } from 'tailwind-merge';
import HelpCircle from '~/ui/icons/lib/help-circle-contained.svg?component';
import { Tooltip } from '../tooltip';
import type { JSXChildren } from '@builder.io/qwik';
import type { Classes } from '~/utils/types';

export type LabelProps = {
  text: string;
  info?: JSXChildren;
  required?: boolean;
  classes?: Classes<'root'>;
};

export const Label = ({ classes, text, info, required }: LabelProps) => (
  <label class={twMerge('w-full flex gap-4 items-center', classes?.root)}>
    <span class="text-space-100 text-sm">
      {text}
      {required ? ' *' : ''}
    </span>
    {info && (
      <Tooltip
        classes={{ root: 'ml-1 cursor-pointer', tooltip: 'text-sm text-space-200' }}
        info={info}
      >
        <HelpCircle />
      </Tooltip>
    )}
  </label>
);
