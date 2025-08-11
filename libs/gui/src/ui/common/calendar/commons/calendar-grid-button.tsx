import { cn } from '~/utils/cn';
import type { PropFunction, QRL } from '@builder.io/qwik';

type CalendarGridButtonProps = {
  class?: string;
  name: string;
  dim?: boolean;
  highlight?: boolean;
  isDisabled?: boolean;
  isSelected: boolean;
  isFocused?: boolean;
  onKeyDown$?: QRL<(event: KeyboardEvent) => void>;
  label: string;
  onSelected$: PropFunction<() => void>;
};

export const CalendarGridButton = (props: CalendarGridButtonProps) => (
  <td
    class={cn(
      'relative p-0 rounded-md text-center text-white text-sm hover:bg-space-800 h-8 w-full',
      props.dim && 'text-space-400',
      props.highlight && 'bg-space-900',
      props.isSelected && 'bg-accent-800 hover:bg-accent-800 text-contrast',
      props.isFocused && 'ring-1 ring-accent-800',
    )}
    tabIndex={-1}
  >
    <button
      disabled={props.isDisabled}
      name="day"
      onKeyDown$={(event) => props.onKeyDown$?.(event)}
      class={cn(
        'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm disabled:pointer-events-none disabled:opacity-50 [&amp;_svg]:pointer-events-none [&amp;_svg]:size-4 [&amp;_svg]:shrink-0 h-full w-full p-0 font-normal',

        props.isDisabled && 'text-gray-500',
      )}
      role="gridcell"
      tabIndex={-1}
      type="button"
      onClick$={() => {
        if (props.isDisabled) return;
        props.onSelected$();
      }}
    >
      {props.label}
    </button>
  </td>
);
