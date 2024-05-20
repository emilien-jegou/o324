import { useId, component$, useSignal } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import type { HTMLAttributes } from '@builder.io/qwik';

type MultiSelectOptionProps = {
  selected?: boolean;
  label: string;
} & HTMLAttributes<HTMLButtonElement>;

export const MultiSelectOption = component$((props: MultiSelectOptionProps) => {
  const id = useId();
  const buttonRef = useSignal<HTMLButtonElement>();

  // eslint-disable-next-line qwik/no-use-visible-task
  //useVisibleTask$(({ track }) => {
  //  track(() => props.selected);
  //  track(() => buttonRef.value);
  //  if (!buttonRef.value || props.selected === false) return;
  //  buttonRef.value.focus();
  //});

  return (
    <button
      ref={buttonRef}
      role="option"
      type="button"
      onClick$={props.onClick$}
      onMouseOver$={props.onMouseOver$}
      aria-labelledby={`radix-:${id}:`}
      aria-selected={props.selected}
      tabIndex={0}
      class={twMerge(
        'flex w-full cursor-pointer select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm outline-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        props.selected && 'select-selected-option bg-space-600',
      )}
      data-radix-collection-item=""
    >
      <span class="absolute right-2 flex h-3.5 w-3.5 items-center justify-center"></span>
      <span id={`radix-:${id}:`}>{props.label}</span>
    </button>
  );
});
