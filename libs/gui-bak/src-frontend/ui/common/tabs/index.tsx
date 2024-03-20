import { twMerge } from 'tailwind-merge';
import { TabButton } from './tab-button';
import type { PropFunction } from '@builder.io/qwik';

type TabsProps<S extends string> = {
  class?: string;
  selected: S;
  options: readonly S[];
  onSelect$: PropFunction<(option: S) => void>;
};

export function Tabs<const S extends string>(props: TabsProps<S>) {
  return (
    <div class={twMerge('flex w-full rounded-sm bg-bg-subtle p-0.5', props.class)}>
      {props.options.map((option, idx) => (
        <TabButton
          key={idx}
          label={option}
          selected={option === props.selected}
          onClick$={() => props.onSelect$(option)}
        />
      ))}
    </div>
  );
}
