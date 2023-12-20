import { $, component$, useId, useSignal, useVisibleTask$ } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { EnterKeyIcon } from '@/ui/icons/enter-key';
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

    const resizeTextArea$ = $((textarea: HTMLTextAreaElement) => {
      if (containerRef.value === undefined) return;

      // Cloning the ref showed the best result when trying to avoid the
      // textarea to flicker.
      const clone = textarea.cloneNode(true) as HTMLTextAreaElement;
      clone.style.height = 'auto';
      clone.style.width = textarea.offsetWidth + 'px';
      clone.style.position = 'absolute';
      clone.style.userSelect = 'none';
      clone.style.left = '200vw';
      clone.style.pointerEvents = 'none';
      clone.style.visibility = 'hidden';

      // We add a character to pre extend the textarea
      clone.value += '0';
      clone.style.zIndex = String(100 + clone.scrollHeight + 5);
      clone.style.overflow = 'hidden';
      document.body.appendChild(clone);
      clone.style.height = clone.scrollHeight + 2 + 'px';
      const computedHeight = clone.scrollHeight;
      textarea.style.height = computedHeight + 2 + 'px';
      containerRef.value.style.height = computedHeight + 'px';

      document.body.removeChild(clone);
    });

    // eslint-disable-next-line qwik/no-use-visible-task
    useVisibleTask$(({ track }) => {
      track(() => value);

      const textarea = document.getElementById(id);
      resizeTextArea$(textarea as any);
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
          'relative rounded-md border border-0.5 border-transparent text-sm leading-none shadow-xs cursor-text bg-bg-subtle',
          focus &&
            'shadow-focused border outline outline-2 outline-focused bg-bg-default border-border-default',
          className,
        )}
      >
        <textarea
          id={id}
          spellcheck={false}
          rows={1}
          class="w-full block h-[39px] p-2.5 pt-3 pr-10 text-sm bg-transparent leading-tight focus:outline-none font-medium placeholder:font-normal text-text-default placeholder:text-text-subtle focus:placeholder:text-subtler resize-none overflow-y-hidden"
          onFocus$={(...args) => onFocus$?.(...args)}
          onBlur$={(event, element) => {
            //resizeTextArea$(event.target as HTMLTextAreaElement);
            onBlur$?.(event, element);
          }}
          onInput$={(event: any) => onInput$?.(event.target.value)}
          value={value}
          {...props}
        />
        <EnterKeyIcon class="text-text-subtle absolute right-4 top-1/2 -translate-y-1/2" />
      </div>
    );
  },
);
