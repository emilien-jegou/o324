import { twMerge } from 'tailwind-merge';
import type { PropFunction } from '@builder.io/qwik';

type TabButtonProps = {
  selected: boolean;
  onClick$?: PropFunction<() => void>;
  label: string;
};

export const TabButton = ({ selected, onClick$, label }: TabButtonProps) => (
  <button
    onClick$={() => onClick$?.()}
    class={twMerge(
      'w-full text-sm p-2 rounded-sm',
      selected ? 'bg-bg-default text-text-default' : 'bg-bg-subtle text-text-subtle',
    )}
  >
    {label}
  </button>
);
