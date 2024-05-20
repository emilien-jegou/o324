import { $, component$, useComputed$, useOnDocument, useSignal } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { FocusCycleController } from '~/ui/logics/focus-cycle-controller';
import { FocusCycleNode } from '~/ui/logics/focus-cycle-node';
import { AutocompleteOption } from './option';
import type { PropFunction, QRL, Signal } from '@builder.io/qwik';

import './styles.css';

export type AutocompleteInputProps = {
  name?: string;
  class?: string;
  'bind:value': Signal<string | undefined>;
  onInput$?: PropFunction<(value: string | undefined) => void>;
  error?: boolean;
  disabled?: boolean;
  placeholder?: string;
  options: string[];
};

export const AutocompleteInput = component$(
  ({ onInput$, options, ...props }: AutocompleteInputProps) => {
    const current = props['bind:value'];
    const ref = useSignal<HTMLDivElement>();
    const inputRef = useSignal<HTMLInputElement>();
    const cyclePosition = useSignal(0);
    const expanded = useSignal(false);

    useOnDocument(
      'click',
      $((e) => {
        if (!ref.value || ref.value.contains(e.target as any)) return;
        expanded.value = false;
      }),
    );

    const filteredOptions = useComputed$(() => {
      const newOptions = options.filter((o) => o !== current.value);
      return newOptions.filter((v) => v.startsWith(current.value ?? ''));
    });

    const select$ = $((value: string) => {
      current.value = value;
      onInput$?.(value);
      if (cyclePosition.value === options.length - current.value.length) {
        cyclePosition.value -= 1;
      }
    });

    const selectCurrent$ = $(() => {
      const elem = filteredOptions.value[cyclePosition.value];
      if (!elem) return;
      select$(elem);
    });

    return (
      <div ref={ref} class={twMerge('relative w-full h-min', props.class)}>
        <FocusCycleController
          bind:position={cyclePosition}
          role="presentation"
          disabled={!expanded.value || !filteredOptions.value.length}
        >
          <input
            ref={inputRef}
            aria-autocomplete="none"
            class={twMerge(
              'field bg-space-900 w-min min-w-[30%] w-full cursor-text flex items-center justify-between whitespace-nowrap rounded-md border border-space-600 bg-transparent px-3 py-2 shadow-sm ring-offset-background [&amp;>span]:line-clamp-1 focus-visible:field-focused',
              props.disabled &&
                'cursor-not-allowed shadow-sm outline-0 border-transparent bg-subtle text-space-300',
              props.error && 'border-error',
            )}
            placeholder={props.placeholder ?? 'select an option'}
            value={current.value ?? ''}
            disabled={props.disabled}
            onFocus$={() => {
              expanded.value = true;
              cyclePosition.value = 0;
            }}
            onInput$={(e) => {
              current.value = (e.target as any)?.value || undefined;
              onInput$?.(current.value);
            }}
            onKeyDown$={async (e) => {
              if (e.key === 'Enter') {
                await selectCurrent$();
                inputRef.value?.blur();
              } else if (e.key === 'Escape') {
                inputRef.value?.blur();
              }
            }}
          />

          {expanded.value && !!filteredOptions.value.length && (
            <div
              role="presentation"
              class={twMerge(
                'select-expanded flex flex-col border border-space-600 shadow-sm rounded-md absolute bg-space-800 z-50 top-[100%] mt-2 p-1 h-fit w-full',
              )}
            >
              {filteredOptions.value.map((option, idx) => (
                <FocusCycleNode
                  key={option}
                  position={idx}
                  render$={$((focused: boolean, focus$: QRL<() => void>) => (
                    <AutocompleteOption
                      onClick$={async (e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        console.info('there');
                        await select$(option);
                        inputRef.value?.blur();
                      }}
                      onMouseOver$={() => focus$()}
                      selected={focused /*option.value === props.selected*/}
                      label={option}
                    />
                  ))}
                />
              ))}
            </div>
          )}
        </FocusCycleController>
      </div>
    );
  },
);
