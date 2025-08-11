import { $, component$, useId } from '@builder.io/qwik';
import { Popover, usePopover } from '@qwik-ui/headless';
import { format } from 'date-fns/format';
import { LibIcon } from '~/ui/icons/lib-icon';
import { cn } from '~/utils/cn';
import { Calendar } from '../calendar';
import type { GenericCalendarProps } from '../calendar';
import type { QRL, Signal } from '@builder.io/qwik';

export type DatePickerInputProps = {
  class?: string;
  ['bind:value']: Signal<Date | undefined>;
  onSelected$?: QRL<(date: Date | undefined) => void>;
} & GenericCalendarProps;

export const DatePickerInput = component$(
  ({ class: className, onSelected$, ...props }: DatePickerInputProps) => {
    const popoverId = useId();
    const { hidePopover } = usePopover(popoverId);

    return (
      <Popover.Root id={popoverId} gutter={8} class={cn('relative', className as string)}>
        <Popover.Trigger>
          <div
            class={cn(
              'field focus-visible:field-focused inline-flex items-center gap-3 whitespace-nowrap rounded-md transition-colors disabled:pointer-events-none disabled:opacity-50 border border-space-600 shadow-sm bg-space-800 hover:bg-space-600 hover:text-white p-2 px-4 w-[240px] justify-start text-left font-normal text-space-400 bg-space-900',
            )}
            aria-haspopup="dialog"
            aria-expanded="false"
          >
            <LibIcon icon="calendar-03" />
            <span class={cn(props['bind:value'].value && 'text-white')}>
              {!props['bind:value'].value
                ? 'Pick a date'
                : format(props['bind:value'].value, 'MMMM do, yyyy')}
            </span>
          </div>
        </Popover.Trigger>
        <Popover.Panel class="bg-transparent">
          <Calendar
            onSelected$={$(async (date: Date | undefined) => {
              await hidePopover();
              onSelected$?.(date);
            })}
            {...props}
          />
        </Popover.Panel>
      </Popover.Root>
    );
  },
);
