import { $, component$, useComputed$, useSignal, useVisibleTask$ } from '@builder.io/qwik';
import { cn } from '~/utils/cn';
import { Views } from './date-picker-helpers';
import type { QRL } from '@builder.io/qwik';

export type TimeModuleProps = {
  currentView: Views;
  selected: Date | undefined;
  onTimeChange$: QRL<(hour: number, min: number) => void>;
};

export const TimeModule = component$((props: TimeModuleProps) => {
  const hours = useComputed$(() => props.selected?.getHours());
  const minutes = useComputed$(() => props.selected?.getMinutes());

  return (
    <div
      style="grid-row: 1;"
      class={cn(
        'relative w-[107px] min-h-[230px] flex-1 overflow-hidden',
        (props.currentView !== Views.Days || !props.selected) && 'pointer-events-none opacity-50',
      )}
    >
      <div
        data-radix-scroll-area-viewport=""
        class="left-0 top-0 pl-2 absolute flex py-2 h-full w-fit rounded-[inherit]"
      >
        <TimeSlider
          class="border-r border-space-800"
          options={Array.from({ length: 24 }, (_, idx) => idx)}
          direction="inverted"
          scrollOnOption={hours.value ?? 12}
          selected={hours.value}
          onSelect$={$((hours: number) => {
            props.onTimeChange$(hours, minutes.value!);
          })}
        />
        <TimeSlider
          options={Array.from({ length: 60 }, (_, idx) => idx)}
          scrollOnOption={minutes.value ?? 30}
          selected={minutes.value}
          onSelect$={$((minutes: number) => {
            props.onTimeChange$(hours.value!, minutes);
          })}
        />
      </div>
    </div>
  );
});

type TimeSliderProps = {
  class?: string;
  options: number[];
  direction?: 'normal' | 'inverted';
  onSelect$: QRL<(option: number) => void>;
  scrollOnOption: number;
  selected?: number;
};

export const TimeSlider = component$((props: TimeSliderProps) => {
  const containerRef = useSignal<HTMLDivElement>();

  const optionsDisplay = useComputed$(() => {
    if (props.direction === 'normal') {
      return props.options;
    } else {
      return props.options.reverse();
    }
  });

  const performScrollToElem$ = $((scrollMode: 'instant' | 'smooth', option: number) => {
    const containerElem = containerRef.value;
    if (!containerElem) {
      throw 'container ref is not set!!';
    }
    let idx = optionsDisplay.value.indexOf(option);

    if (idx === -1) {
      idx = optionsDisplay.value.length / 2;
    }

    const button = containerElem.querySelector<HTMLButtonElement>(`[data-option-index="${idx}"]`);
    if (!button) {
      console.error('TimeSlider - scroll prevented due to missing option');
      return;
    }
    button.scrollIntoView({ behavior: scrollMode, block: 'center' });
  });

  // eslint-disable-next-line qwik/no-use-visible-task
  useVisibleTask$(({ track }) => {
    track(() => props.selected);
    performScrollToElem$('instant', props.scrollOnOption);
  });

  return (
    <div
      tabIndex={props.selected ? 1 : -1}
      ref={containerRef}
      onKeyDown$={(event) => {
        if (props.selected === undefined) return;
        if (event.key === 'ArrowRight') {
          const idx = optionsDisplay.value.indexOf(props.selected);
          if (idx === -1) return;
          const fidx = (optionsDisplay.value.length + idx + 1) % optionsDisplay.value.length;
          console.info(fidx);
          props.onSelect$(optionsDisplay.value[fidx]);
        } else if (event.key === 'ArrowLeft') {
          const idx = optionsDisplay.value.indexOf(props.selected);
          if (idx === -1) return;
          const fidx = (optionsDisplay.value.length + idx - 1) % optionsDisplay.value.length;
          props.onSelect$(optionsDisplay.value[fidx]);
        } else if (event.key === 'Enter') {
          const focusableElements = document.querySelectorAll('[tabindex]:not([tabindex="-1"])');
          let currentIndex = -1;

          // Find the currently focused element
          focusableElements.forEach((el, index) => {
            if (el === document.activeElement) {
              currentIndex = index;
            }
          });

          // Focus the next element (loop back to the start if necessary)
          if (focusableElements.length > 0) {
            const nextIndex = (currentIndex + 1) % focusableElements.length;
            (focusableElements[nextIndex] as any).focus();
          }
        }
      }}
      class={cn(
        'overflow-scroll slim-scrollbar px-2 flex flex-col w-fit',
        !props.selected && 'pointer-events-none',
        props.class,
      )}
    >
      {optionsDisplay.value.map((opt, idx) => (
        <option
          key={opt}
          data-option-index={idx}
          tabIndex={-1}
          class={cn(
            'cursor-pointer inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm text-space-100 font-medium disabled:pointer-events-none disabled:opacity-50 hover:bg-space-800 hover:text-white h-8 w-8 shrink-0 aspect-square',
            props.selected === opt && 'bg-accent-800 text-space-1000',
          )}
          onClick$={() => {
            performScrollToElem$('smooth', opt);
            props.onSelect$(opt);
          }}
        >
          {String(opt)}
        </option>
      ))}
    </div>
  );
});
