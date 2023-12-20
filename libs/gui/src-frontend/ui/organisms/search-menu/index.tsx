import { $, component$, useOnWindow, useSignal, useTask$ } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { SearchInput } from '@/ui/common/search-input';

type SearchMenuProps = {
  class: string;
};

export const SearchMenu = component$((props: SearchMenuProps) => {
  const isFocused = useSignal(false);
  const input = useSignal('');

  const setFocusCurried = (value: boolean) =>
    $(() => {
      isFocused.value = value;
    });

  useTask$(({ track }) => {
    track(() => isFocused.value);
    if (isFocused.value === false) return;
  });

  useOnWindow(
    'keydown',
    $((event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        isFocused.value = false;
        input.value = '';
      } else if (event.key === 'Enter') {
        isFocused.value = true;
      }
    }),
  );

  return (
    <div class={props.class}>
      <SearchInput
        value={input.value}
        class="relative z-50"
        onInput$={(value) => {
          input.value = value;
        }}
        onFocus$={setFocusCurried(true)}
        placeholder="What are you working on ?"
        focus={isFocused.value}
      />
      <div
        onClick$={(e) => e.preventDefault()}
        class={twMerge(
          'absolute top-0 left-0 z-20 o3-h-screen w-screen bg-bg-subtle transition-opacity duration-[40ms]',
          !isFocused.value && 'pointer-events-none opacity-0',
        )}
      >
        {/* TODO */}
      </div>
    </div>
  );
});
