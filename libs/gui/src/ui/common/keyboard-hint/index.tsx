import { cn } from '~/utils/cn';
import type { JSXChildren } from '@builder.io/qwik';

type KeyboardHintProps = {
  hint: JSXChildren;
  class: string;
};

export const KeyboardHint = (props: KeyboardHintProps) => (
  <div class={cn('flex items-center justify-center p-1 rounded-sm', props.class)}>{props.hint}</div>
);
