import { twMerge } from 'tailwind-merge';
import { HelpCircleIcon } from '~/ui/icons/help-circle-icon';
import { Tooltip } from '../tooltip';
import type { TooltipPosition } from '../tooltip';
import type { JSXChildren } from '@builder.io/qwik';
import type { Classes } from '~/utils/types';

export type LabelProps = {
  text: string;
  info?: JSXChildren;
  tooltipPosition?: TooltipPosition;
  required?: boolean;
  classes?: Classes<'root'>;
};

export const Label = ({ classes, text, info, required, tooltipPosition }: LabelProps) => (
  <label class={twMerge('flex justify-between w-full flex gap-4 items-center', classes?.root)}>
    <span class="text-space-100 text-sm">
      {text}
      {required ? ' *' : ''}
    </span>
    {info && (
      <Tooltip
        classes={{
          root: 'ml-1 mr-1 text-space-400 transition-all cursor-pointer hover:text-accent-100',
        }}
        position={tooltipPosition}
        info={info}
      >
        <HelpCircleIcon class="w-5 h-5" />
      </Tooltip>
    )}
  </label>
);
