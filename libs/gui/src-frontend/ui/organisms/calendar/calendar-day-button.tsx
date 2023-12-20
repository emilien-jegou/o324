import { twMerge } from 'tailwind-merge';

// TODO: move this
type CalendarDayButtonProps = {
  date: Date;
  selected: boolean;
  disabled: boolean;
};

export const CalendarDayButton = ({ date, selected, disabled }: CalendarDayButtonProps) => {
  const dayAbbreviation = date.toLocaleString('en-US', { weekday: 'short' });
  const dayOfMonth = date.getDate();

  return (
    <button
      class={twMerge(
        'flex pt-2 pb-1.5 px-1 rounded-full flex-col items-center text-text-default',
        selected && 'bg-bg-subtle',
      )}
      disabled={disabled}
    >
      <span class={twMerge('text-xs text-text-subtle', disabled && 'text-text-disabled')}>
        {dayAbbreviation}
      </span>
      <span class={twMerge('font-semibold', disabled && 'text-text-disabled')}>{dayOfMonth}</span>
    </button>
  );
};
