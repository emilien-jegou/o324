import { $, component$, useId, useOnDocument, useSignal } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { SelectExpandIcon } from '~/ui/icons/select-expand';
import { FocusCycleController } from '~/ui/logics/focus-cycle-controller';
import { FocusCycleNode } from '~/ui/logics/focus-cycle-node';
import { findIndex } from '~/utils/safe-std';
import { SelectOption } from './option';
import type { PropFunction, QRL } from '@builder.io/qwik';

import './styles.css';

export type SelectProps = {
  name?: string;
  class?: string;
  selected?: string;
  onSelect$?: PropFunction<(v: string) => void>;
  error?: boolean;
  disabled?: boolean;
  placeholder?: string;
  options: { value: string; label: string }[];
};

export const Select = component$((props: SelectProps) => {
  const ref = useSignal<HTMLDivElement>();
  const buttonRef = useSignal<HTMLButtonElement>();
  const focused = useSignal(false);
  const expanded = useSignal(false);
  const cyclePosition = useSignal(findIndex(props.options, (o) => o.value === props.selected) ?? 0);
  const id = useId();

  useOnDocument(
    'click',
    $((e) => {
      if (!ref.value || ref.value.contains(e.target as any)) return;
      expanded.value = false;
    }),
  );

  return (
    <div ref={ref} class={twMerge('relative w-full', props.class)}>
      <button
        ref={buttonRef}
        type="button"
        role="combobox"
        aria-controls={`radix-:${id}:`}
        aria-expanded={expanded.value}
        disabled={props.disabled}
        onFocus$={() => {
          focused.value = true;
        }}
        onBlur$={() => {
          focused.value = false;
        }}
        onClick$={(e) => {
          e.preventDefault();
          e.stopPropagation();
          expanded.value = !expanded.value;
        }}
        aria-autocomplete="none"
        class={twMerge(
          'field w-full flex items-center justify-between whitespace-nowrap rounded-md border border-space-600 bg-transparent px-3 py-2 shadow-sm ring-offset-background [&amp;>span]:line-clamp-1',
          (expanded.value || focused.value) && 'field-accent-500',
          props.disabled &&
            'cursor-not-allowed shadow-sm outline-0 border-transparent bg-subtle text-space-300',
          props.error && 'border-error',
        )}
      >
        <span class={twMerge('pointer-events-none', !props.selected && 'text-space-300')}>
          {(props.selected ? props.options.find((a) => a.value === props.selected)?.label : null) ??
            props.placeholder ??
            'select an option'}
        </span>
        <SelectExpandIcon class="h-4 w-4 opacity-50" />
      </button>

      {expanded.value && (
        <FocusCycleController
          bind:position={cyclePosition}
          role="presentation"
          class={twMerge(
            'select-expanded flex flex-col border border-space-600 shadow-sm rounded-md absolute bg-space-800 z-50 top-[100%] mt-2 p-1 h-fit w-full',
          )}
        >
          {props.options.map((option, idx) => (
            <FocusCycleNode
              key={idx}
              position={idx}
              render$={$((focused: boolean, focus$: QRL<() => void>) => (
                <SelectOption
                  onClick$={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    expanded.value = false;
                    props.onSelect$?.(option.value);
                    buttonRef.value?.focus();
                  }}
                  onMouseOver$={() => focus$()}
                  selected={focused /*option.value === props.selected*/}
                  label={option.label}
                />
              ))}
            />
          ))}
        </FocusCycleController>
      )}
    </div>
  );
});
