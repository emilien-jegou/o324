import { twMerge } from 'tailwind-merge';
import type { JSXChildren } from '@builder.io/qwik';

type KeyboardHintProps = {
  hint: JSXChildren;
  class: string;
};

export const KeyboardHint = (props: KeyboardHintProps) => (
  <div class={twMerge('flex items-center justify-center p-1 rounded-sm', props.class)}>
    {props.hint}
  </div>
);
