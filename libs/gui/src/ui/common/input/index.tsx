import { component$, useId } from '@builder.io/qwik';
import { cn } from '~/utils/cn';
import type { PropFunction, QwikFocusEvent, Signal } from '@builder.io/qwik';
import type { FieldElement } from '@modular-forms/qwik';

type RefFnInterface<EL> = {
  (el: EL): void;
};

type Ref<EL extends Element = Element> = Signal<Element | undefined> | RefFnInterface<EL>;

export type InputType = 'text' | 'password';

export type InputAutocomplete = 'email' | 'new-password' | 'current-password';

export type InputProps = {
  autoFocus?: boolean;
  autocomplete?: InputAutocomplete;
  disabled?: boolean;
  name: string;
  onBlur$?: PropFunction<(event: QwikFocusEvent<FieldElement>, element: FieldElement) => void>;
  onFocus$?: PropFunction<(event: QwikFocusEvent<FieldElement>, element: FieldElement) => void>;
  onChange$?: PropFunction<(value: string) => void>;
  onInput$?: PropFunction<(value: string) => void>;
  ref?: Ref<HTMLInputElement> | undefined;
  placeholder?: string;
  type?: InputType;
  value?: string;
  error?: boolean;
  class?: string;
  readOnly?: boolean;
  tabIndex?: number;
};

export const Input = component$(
  ({
    disabled,
    class: className,
    error,
    onBlur$,
    onFocus$,
    onChange$,
    onInput$,
    tabIndex,
    ...props
  }: InputProps) => {
    const id = useId();

    return (
      <input
        id={id}
        tabIndex={tabIndex ?? 1}
        disabled={disabled}
        onInput$={(e: any) => onInput$?.(e.target.value)}
        onChange$={(e: any) => onChange$?.(e.target.value)}
        onBlur$={(...args) => {
          onBlur$?.(...args);
        }}
        onFocus$={(...args) => {
          onFocus$?.(...args);
        }}
        class={cn(
          'bg-space-900 field border border-space-600 transition-outline rounded-md p-2 w-full focus-visible:field-focused',
          disabled &&
            'cursor-not-allowed shadow-sm outline-0 border-transparent bg-subtle text-space-300',
          error && 'border-red-500',
          className,
        )}
        {...props}
      />
    );
  },
);
