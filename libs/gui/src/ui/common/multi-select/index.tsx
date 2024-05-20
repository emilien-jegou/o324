import { $, component$, useComputed$, useId, useOnDocument, useSignal, useTask$ } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { SelectExpandIcon } from '~/ui/icons/select-expand';
import { FocusCycleController } from '~/ui/logics/focus-cycle-controller';
import { FocusCycleNode } from '~/ui/logics/focus-cycle-node';
import { MultiSelectOption } from './option';
import type { JSXOutput, PropFunction, QRL, Signal } from '@builder.io/qwik';

import './styles.css';

type RenderEmptyArgs = {
  searchValue: Signal<string | undefined>;
  select$: QRL<(value: string) => void>;
};

export type MultiSelectOption = { value: string; label: string };

export type MultiSelectProps = {
  name?: string;
  class?: string;
  'bind:value': Signal<string[]>;
  onSelect$?: PropFunction<(v: string) => void>;
  onUnselect$?: PropFunction<(v: string) => void>;
  onSearchInput$?: PropFunction<(value: string | undefined) => void>;
  error?: boolean;
  disabled?: boolean;
  placeholder?: string;
  options: MultiSelectOption[];
  renderEmpty$?: QRL<(args: RenderEmptyArgs) => JSXOutput>;
};

export const MultiSelect = component$(
  ({
    renderEmpty$,
    onSearchInput$,
    options,
    onSelect$,
    onUnselect$,
    ...props
  }: MultiSelectProps) => {
    const current = props['bind:value'];
    const ref = useSignal<HTMLDivElement>();
    const buttonRef = useSignal<HTMLButtonElement>();
    const inputRef = useSignal<HTMLInputElement>();
    const searchValue = useSignal<string | undefined>();
    const focused = useSignal(false);
    const cyclePosition = useSignal(0);
    const expanded = useSignal(false);
    const id = useId();

    useOnDocument(
      'click',
      $((e) => {
        if (!ref.value || ref.value.contains(e.target as any)) return;
        expanded.value = false;
      }),
    );

    useTask$(({ track }) => {
      track(() => options);
      console.info('OPT', options);
    });

    const filteredOptions = useComputed$(() => {
      const currentSet = new Set(current.value);
      const newOptions = options.filter((o) => !currentSet.has(o.value));
      return newOptions.filter((v) =>
        v.label.toLowerCase().startsWith((searchValue.value ?? '').toLowerCase()),
      );
    });

    const select$ = $((value: string) => {
      current.value = [...new Set([...current.value, value])];
      searchValue.value = '';
      onSelect$?.(value);
      if (cyclePosition.value === options.length - current.value.length) {
        cyclePosition.value -= 1;
      }
    });

    const unselect$ = $((value: string) => {
      current.value = current.value.filter((v) => v !== value);
      onUnselect$?.(value);
    });

    const selectCurrent$ = $(() => {
      const elem = filteredOptions.value[cyclePosition.value];
      select$(elem.value);
    });

    const emptyElement = useComputed$(() => {
      return renderEmpty$?.({ select$, searchValue });
    });

    return (
      <div ref={ref} class={twMerge('relative w-full h-min', props.class)}>
        <FocusCycleController
          bind:position={cyclePosition}
          role="presentation"
          disabled={!expanded.value}
        >
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
              if (expanded.value) {
                cyclePosition.value = 0;
                inputRef.value?.focus();
              } else {
                inputRef.value?.blur();
              }
            }}
            aria-autocomplete="none"
            class={twMerge(
              'field w-full cursor-text flex items-center justify-between whitespace-nowrap rounded-md border border-space-600 bg-transparent px-3 py-2 shadow-sm ring-offset-background [&amp;>span]:line-clamp-1',
              (expanded.value || focused.value) && 'field-accent-500',
              props.disabled &&
              'cursor-not-allowed shadow-sm outline-0 border-transparent bg-subtle text-space-300',
              props.error && 'border-error',
            )}
          >
            <div class="flex flex-wrap items-center gap-1 w-full">
              {current.value.map((v) => (
                <button
                  key={v}
                  type="button"
                  tabIndex={-1}
                  onClick$={(e) => {
                    e.stopPropagation();
                    unselect$(v);
                    inputRef.value?.focus();
                  }}
                  class="inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none border-transparent bg-space-600 text-primary-foreground hover:bg-space-500"
                >
                  {options.find((o) => o.value === v)?.label}
                </button>
              ))}
              <input
                ref={inputRef}
                tabIndex={-1}
                class="bg-space-900 pointer-events-none focus:outline-none w-min text-base min-w-[30%]"
                placeholder={props.placeholder ?? 'select an option'}
                value={searchValue.value}
                onInput$={(e) => {
                  searchValue.value = (e.target as any)?.value || undefined;
                  onSearchInput$?.(searchValue.value);
                }}
                onKeyDown$={(e) => {
                  if (e.key === 'Enter') {
                    selectCurrent$();
                    e.stopPropagation();
                    e.preventDefault();
                  }
                  else if (e.key === 'Delete' || e.key === 'Backspace') {
                    if (!searchValue.value?.length && current.value.length > 0) {
                      unselect$(current.value[current.value.length - 1]);
                    }
                  } else if (e.key === 'Escape') {
                    searchValue.value = '';
                    focused.value = false;
                    expanded.value = false;
                    inputRef.value?.blur();
                  }
                }}
                onBlur$={() => {
                  searchValue.value = undefined;
                }}
              />
            </div>
            <SelectExpandIcon class="h-4 w-4 cursor-pointer opacity-50" />
          </button>

          {expanded.value && (
            <div
              role="presentation"
              onFocusIn$={(e) => {
                e.preventDefault();
                inputRef.value?.focus();
              }}
              class={twMerge(
                'select-expanded flex flex-col border border-space-600 shadow-sm rounded-md absolute bg-space-800 z-50 top-[100%] mt-2 p-1 h-fit w-full max-h-[160px] overflow-y-auto',
              )}
            >
              {filteredOptions.value.length === 0 &&
                (emptyElement.value ? (
                  emptyElement.value
                ) : (
                  <div class="py-1 text-sm text-space-400 text-center">no option available</div>
                ))}
              {filteredOptions.value.map((option, idx) => (
                <FocusCycleNode
                  key={option.value}
                  position={idx}
                  render$={$((focused: boolean, focus$: QRL<() => void>) => (
                    <MultiSelectOption
                      onClick$={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        select$(option.value);
                      }}
                      onMouseOver$={() => focus$()}
                      selected={focused /*option.value === props.selected*/}
                      label={option.label}
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
