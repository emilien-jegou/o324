import { twMerge } from 'tailwind-merge';
import type { JSXChildren } from '@builder.io/qwik';

type PaneProps = {
  index: number;
  currentIndex: number;
  children: JSXChildren;
};

export const Pane = (props: PaneProps) => (
  <div
    style={{ transform: `translateX(${(props.index - props.currentIndex) * 100}%)` }}
    class={twMerge(
      'transition absolute top-0 left-0 w-full h-full duration-200',
      props.index !== props.currentIndex && 'select-none pointer-events-none',
    )}
  >
    {props.children}
  </div>
);
