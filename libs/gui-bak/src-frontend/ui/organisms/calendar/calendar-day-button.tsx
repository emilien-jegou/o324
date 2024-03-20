import { twMerge } from 'tailwind-merge';
import type { PropFunction } from '@builder.io/qwik';

// TODO: move this
type CalendarDayButtonProps = {
  date: Date;
  selected: boolean;
  disabled: boolean;
  onClick$?: PropFunction<() => void>;
};

export const CalendarDayButton = ({
  date,
  selected,
  disabled,
  onClick$,
}: CalendarDayButtonProps) => {
  const dayAbbreviation = date.toLocaleString('en-US', { weekday: 'short' }).slice(0, 2);
  const dayOfMonth = date.getDate();

  return (
    <button
      class={twMerge(
        'flex select-none pt-2 pb-1.5 px-1 rounded-full flex-col items-center text-text-default',
        selected && 'bg-bg-subtle',
      )}
      onClick$={() => onClick$?.()}
      disabled={disabled}
    >
      <span class={twMerge('text-xs text-text-subtle', disabled && 'text-text-disabled')}>
        {dayAbbreviation}
      </span>
      <span class={twMerge('font-semibold', disabled && 'text-text-disabled')}>{dayOfMonth}</span>
    </button>
  );
};
