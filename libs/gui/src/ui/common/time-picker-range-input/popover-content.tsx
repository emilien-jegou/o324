import { $, component$, useSignal } from '@builder.io/qwik';
import { useFocusTrap } from '~/hooks/use-focus-trap';
import { Calendar } from '../calendar';
import { Tabs } from '../tabs';
import type { TimePickerValue } from '.';
import type { QRL, Signal } from '@builder.io/qwik';

export type PopoverContentProps = {
  ['bind:value']: Signal<TimePickerValue>;
  onClose$: QRL<() => void>;
};

type View = 'start' | 'end';

export const PopoverContent = component$((props: PopoverContentProps) => {
  const ref = useSignal<HTMLDivElement | undefined>();
  const value = props['bind:value'];
  const currentView = useSignal<View>('start');
  const currentDate = useSignal<Date | undefined>(value.value.start);

  useFocusTrap(ref);

  return (
    <div ref={ref} class="bg-space-1000 p-2 rounded-md border border-space-700 w-fit">
      <Tabs
        selected={currentView}
        onSelect$={$((view: View) => {
          currentView.value = view;
          if (view === 'end') {
            console.info('in here');
            currentDate.value = value.value.end;
          } else {
            currentDate.value = value.value.start;
          }
        })}
        options={[
          { label: 'Start', value: 'start' },
          { label: 'End', value: 'end' },
        ]}
      />
      <Calendar
        bind:value={currentDate}
        mode="date-and-time"
        onSelected$={$((newDate: Date | undefined) => {
          if (currentView.value === 'start') {
            value.value = { ...value.value, start: newDate };
          } else {
            value.value = { ...value.value, end: newDate };
          }
        })}
      />
      <div class="flex items-center gap-2 mx-[2px] mt-2">
        <TimeInput />
        <button
          tabIndex={1}
          onClick$={$(() => {
            props.onClose$();
          })}
          class="border p-1 border-space-800 transition-colors hover:bg-space-700 hover:text-white text-space-400 rounded-md w-full"
        >
          Close
        </button>
      </div>
    </div>
  );
});

type TimeInputProps = {
  //onChange$: (newValue: string) => void;
};

export const TimeInput = component$((props: TimeInputProps) => {
  const mode = useSignal<'button' | 'input'>();

  return (
    <div class="w-full cursor-text hover:border border-space-700 p-1 rounded-md hover:underline text-white text-center">
      00:34:00
    </div>
  );
});
