import { $, component$, useId, useOn, useOnDocument, useSignal } from '@builder.io/qwik';
import { Popover, usePopover } from '@qwik-ui/headless';
import { isSameDay } from 'date-fns';
import { format } from 'date-fns/format';
import { useClickOutside } from '~/hooks/use-click-outside';
import { LibIcon } from '~/ui/icons/lib-icon';
import { cn } from '~/utils/cn';
import { PopoverContent } from './popover-content';
import type { QRL, Signal } from '@builder.io/qwik';

export type TimePickerValue = { start: Date | undefined; end: Date | undefined };

export type TimePickerRangeInputProps = {
  class?: string;
  ['bind:value']: Signal<TimePickerValue>;
  onSelected$?: QRL<(date: Date | undefined) => void>;
};

export const TimePickerRangeInput = component$(
  ({ class: className, onSelected$, ...props }: TimePickerRangeInputProps) => {
    const popoverId = useId();
    const { showPopover, hidePopover } = usePopover(popoverId);
    const panelRef = useSignal<HTMLElement | undefined>();
    const rootRef = useSignal<HTMLElement | undefined>();

    // TODO: this should be disabled when a modal is opened
    useOnDocument(
      'keypress',
      $((event: KeyboardEvent) => {
        if (event.key === 'Escape') {
          hidePopover();
        }
      }),
    );

    useClickOutside(
      [rootRef, panelRef],
      $(() => {
        hidePopover();
      }),
    );

    return (
      <Popover.Root
        ref={rootRef}
        manual
        id={popoverId}
        gutter={8}
        class={cn('relative', className as string)}
      >
        <Popover.Trigger tabIndex={1}>
          <div
            onClick$={() => showPopover()}
            class={cn(
              'field inline-flex items-center gap-3 whitespace-nowrap rounded-md transition-colors disabled:pointer-events-none disabled:opacity-50 border border-space-600 shadow-sm bg-space-800 hover:bg-space-600 hover:text-white p-2 px-4 w-[240px] justify-start text-left font-normal text-space-400 bg-space-900',
            )}
            aria-haspopup="dialog"
            aria-expanded="false"
          >
            <LibIcon icon="calendar-03" />
            <span class={cn(props['bind:value'].value.start && 'text-white')}>
              {!props['bind:value'].value.start && !props['bind:value'].value.end ? (
                'Pick a date'
              ) : (
                <PickedLabel value={props['bind:value'].value} />
              )}
            </span>
          </div>
        </Popover.Trigger>
        <Popover.Panel ref={panelRef} class="bg-transparent">
          <PopoverContent
            bind:value={props['bind:value']}
            onClose$={$(async () => {
              await hidePopover();
            })}
          />
        </Popover.Panel>
      </Popover.Root>
    );
  },
);

export type PickedLabelProps = {
  value: TimePickerValue;
};

export const PickedLabel = (props: PickedLabelProps) => {
  return (
    <div class="flex gap-2 items-center">
      <span>{props.value.start ? format(props.value.start, 'dd-MM, HH:mm') : '-'}</span>
      <LibIcon icon="arrow-right" />
      <span>{props.value.end ? endDateFormatter(props.value.start, props.value.end) : '-'}</span>
    </div>
  );
};

const endDateFormatter = (start: Date | undefined, end: Date) => {
  if (start !== undefined && isSameDay(start, end)) {
    return format(end, 'HH:mm');
  } else {
    return format(end, 'dd-MM, HH:mm');
  }
};
