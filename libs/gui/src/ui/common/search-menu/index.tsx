import { $, component$, useOnDocument, useSignal } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { useClickOutside } from '~/hooks/use-click-outside';
import { SearchInput } from '~/ui/common/search-input';
import type { PropFunction, FunctionComponent } from '@builder.io/qwik';

type SearchMenuProps = {
  class: string;
  placeholder: string;
  MenuContent: FunctionComponent<{ value: string; onSubmit$: PropFunction<() => void> }>;
  onSelect$?: PropFunction<(value: string) => void>;
};

export const SearchMenu = component$(({ MenuContent, ...props }: SearchMenuProps) => {
  const onSelect$ = props.onSelect$;
  const isFocused = useSignal(false);
  const containerRef = useSignal<HTMLFormElement>();
  const input = useSignal('');

  // TODO: this should be disabled when a modal is opened
  useOnDocument(
    'keypress',
    $((event: KeyboardEvent) => {
      const current = isFocused.value;
      if (event.key === 'Escape') {
        isFocused.value = false;
        input.value = '';
      } else if (event.key === 'Enter') {
        if (current === false) {
          isFocused.value = true;
        }
      }
    }),
  );

  useClickOutside(
    containerRef,
    $(() => {
      isFocused.value = false;
      input.value = '';
    }),
  );
  const submit = $(() => {
    onSelect$?.(input.value);
    isFocused.value = false;
    input.value = '';
  });

  return (
    <form
      ref={containerRef}
      noValidate={true}
      preventdefault:submit
      onSubmit$={submit}
      class={twMerge('relative', props.class)}
    >
      <SearchInput
        value={input.value}
        class="relative text-white z-50"
        onInput$={(value) => {
          input.value = value;
        }}
        onFocus$={$(() => {
          isFocused.value = true;
        })}
        onBlur$={$(() => {
          // TODO: this prevent paste on right click
          if (input.value.trim().length === 0) {
            isFocused.value = false;
          }
        })}
        placeholder={props.placeholder}
        focus={isFocused.value}
      />
      <div
        onClick$={(e) => e.preventDefault()}
        key={input.value}
        class={twMerge(
          'absolute top-[100%] left-0 z-20 w-full transition-opacity duration-40',
          !isFocused.value && 'opacity-0',
        )}
      >
        <div
          class={twMerge(
            'w-full h-full rounded-sm rounded-t-none bg-space-800 border border-space-700 shadow-xl',
            !isFocused.value && 'opacity-0',
          )}
        >
          <MenuContent value={input.value} onSubmit$={$(() => submit())} />
        </div>
      </div>
    </form>
  );
});
