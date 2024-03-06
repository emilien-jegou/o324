import { twMerge } from 'tailwind-merge';
import type { JSXChildren, PropFunction } from '@builder.io/qwik';

type IconButtonProps = {
  onClick$?: PropFunction<() => void>;
  icon: JSXChildren;
};

export const IconButton = (props: IconButtonProps) => (
  <button class="p-2 hover:bg-contrast/10 text-black" onClick$={props.onClick$}>
    {props.icon}
  </button>
);
