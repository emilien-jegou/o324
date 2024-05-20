import { $, component$, useId, useSignal, useVisibleTask$ } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { EnterKeyIcon } from '~/ui/icons/enter-key';
import { KeyboardHint } from '../keyboard-hint';
import type { PropFunction, QwikFocusEvent } from '@builder.io/qwik';
import type { FieldElement } from '@modular-forms/qwik';

export type SearchInputProps = {
  onFocus$?: PropFunction<(event: QwikFocusEvent<FieldElement>, element: FieldElement) => void>;
  onBlur$?: PropFunction<(event: QwikFocusEvent<FieldElement>, element: FieldElement) => void>;
  onInput$?: PropFunction<(value: string) => void>;
  placeholder?: string;
  value?: string;
  class?: string;
  focus?: boolean;
};

export const SearchInput = component$(
  ({ class: className, focus, onBlur$, onFocus$, onInput$, value, ...props }: SearchInputProps) => {
    const id = useId();
    const containerRef = useSignal<HTMLDivElement>();

    const focusInput = $(() => {
      setTimeout(() => {
        document.getElementById(id)?.focus();
      });
    });

    // eslint-disable-next-line qwik/no-use-visible-task
    useVisibleTask$(({ track }) => {
      track(() => focus);
      if (focus === false) {
        document.getElementById(id)?.blur();
      } else {
        document.getElementById(id)?.focus();
      }
    });

    return (
      <div
        onMouseDown$={focusInput}
        onClick$={focusInput}
        ref={containerRef}
        style={{
          transition: 'height 0.1s ease',
        }}
        class={twMerge(
          'relative text-sm leading-none shadow-xs cursor-text border border-space-700 rounded-sm bg-space-800',
          focus && 'outline outline-2 outline-focused rounded-b-none',
          className,
        )}
      >
        <input
          id={id}
          name="search-input"
          spellcheck={false}
          class="w-full block h-[39px] p-2.5 pt-3 pr-10 text-sm bg-transparent leading-tight focus:outline-none font-medium placeholder:font-normal text-white placeholder:text-space-400 focus:placeholder:text-space-600 resize-none overflow-y-hidden"
          onFocus$={(...args) => onFocus$?.(...args)}
          onBlur$={(event, element) => {
            onBlur$?.(event, element);
          }}
          onInput$={(event: any) => onInput$?.(event.target.value)}
          value={value}
          {...props}
        />
        <KeyboardHint
          class={twMerge(
            'text-space-200 absolute right-4 top-1/2 -translate-y-1/2 transition-opacity',
            focus && 'opacity-0',
          )}
          hint={<EnterKeyIcon />}
        />
      </div>
    );
  },
);
