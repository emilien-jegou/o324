import type { Signal } from '@builder.io/qwik';
import { $, type QRL } from '@builder.io/qwik';

type TabsProps<T> = {
  ref?: Signal<HTMLDivElement | undefined>;
  options: { value: T; label: string }[];
  onSelect$: QRL<(value: T) => void>;
  onKeyDown$?: QRL<(value: KeyboardEvent) => void>;
  selected: Signal<T>;
};

export const Tabs = function <T>({ ref, onSelect$, onKeyDown$, options, selected }: TabsProps<T>) {
  return (
    <div
      ref={ref}
      role="tablist"
      aria-orientation="horizontal"
      class="relative mb-2 p-1 font-medium items-center justify-center rounded-md bg-space-900 text-space-500 grid w-full grid-cols-2"
      tabIndex={1}
      onKeyDown$={(event) => {
        if (event.key === 'ArrowRight') {
          const idx = options.findIndex((o) => o.value === selected.value);

          if (idx === -1) {
            return;
          }
          const newIdx = (options.length + idx + 1) % options.length;
          const newValue = options[newIdx];
          onSelect$(newValue.value);
        } else if (event.key === 'ArrowLeft') {
          const idx = options.findIndex((o) => o.value === selected.value);

          if (idx === -1) {
            return;
          }
          const newIdx = (options.length + idx + 1) % options.length;
          const newValue = options[newIdx];
          onSelect$(newValue.value);
        }

        onKeyDown$?.(event);
      }}
      data-orientation="horizontal"
      style="outline: none;"
    >
      {options.map(({ value, label }, idx) => (
        <Tab
          key={idx}
          selected={value === selected.value}
          label={label}
          onSelect$={$(() => onSelect$(value))}
        />
      ))}
    </div>
  );
};

type TabProps = {
  label: string;
  onSelect$: QRL<() => void>;
  selected: boolean;
};

export const Tab = (props: TabProps) => (
  <button
    type="button"
    role="tab"
    aria-selected={props.selected}
    aria-controls="radix-:re:-content-account"
    data-selected={String(props.selected)}
    id="radix-:re:-trigger-account"
    class="inline-flex items-center justify-center whitespace-nowrap rounded-md px-2 py-1.5 text-sm transition-all data-[selected=true]:bg-space-1000 data-[selected=true]:text-white data-[selected=true]:shadow-sm hover:text-white ring-0 outline-none focus:ring-0"
    tabIndex={-1}
    data-orientation="horizontal"
    onClick$={props.onSelect$}
    data-radix-collection-item=""
  >
    {props.label}
  </button>
);
