import { component$, useSignal } from '@builder.io/qwik';
import { cn } from '~/utils/cn';

export const TimePickerInput = component$(() => {
  const hours = useSignal(0);
  const mins = useSignal(0);
  const secs = useSignal(0);
  const focused = useSignal(false);

  return (
    <button
      class="w-full flex gap-1 py-1.5 px-4 items-center border border-space-600 rounded-md hover:bg-space-600 text-white"
      type="button"
      tabIndex={1}
      onClick$={() => {
        focused.value = true;
      }}
    >
      <TimePickerInputVal
        class={focused.value === false ? 'pointer-events-none' : undefined}
        name="hours"
        value={hours.value}
      />
      <span>:</span>
      <TimePickerInputVal
        class={focused.value === false ? 'pointer-events-none' : undefined}
        name="hours"
        value={mins.value}
      />
      <span>:</span>
      <TimePickerInputVal
        class={focused.value === false ? 'pointer-events-none' : undefined}
        name="hours"
        value={secs.value}
      />
    </button>
  );
});

type TimePickerInputValProps = {
  name: string;
  value: number;
  class?: string;
};

const TimePickerInputVal = (props: TimePickerInputValProps) => (
  <input
    type="tel"
    class={cn(
      'flex rounded-md py-1 w-[24px] ring-offset-background text-center font-mono text-base tabular-nums caret-transparent bg-transparent focus:bg-space-600 focus:text-accent-foreground [&amp;::-webkit-inner-spin-button]:appearance-none',
      props.class,
    )}
    id="hours"
    inputMode="decimal"
    name={props.name}
    value={String(props.value).padStart(2, '0')}
  />
);
