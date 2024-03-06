import { twMerge } from 'tailwind-merge';
import type { JSXChildren, PropFunction } from '@builder.io/qwik';

export type ButtonProps = {
  disabled?: boolean;
  label: string;
  iconEnd?: JSXChildren;
  iconStart?: JSXChildren;
  class?: string;
  type?: 'button' | 'submit';
  onClick$?: PropFunction<() => void>;
};

export const Button = (props: ButtonProps) => (
  <button
    class={twMerge(
      'flex flex-row justify-center items-center whitespace-nowrap rounded-sm gap-2 h-10  font-medium bg-bg-default border border-[0.5px] border-border-subtler hover:drop-shadow-sm text-text-default',
      'text-xs px-4 py-1', // md
      props.disabled && 'cursor-hand bg-bg-subtle text-text-subtle',
      props.class,
    )}
    disabled={props.disabled}
    type={props.type ?? 'button'}
    onClick$={props.onClick$}
  >
    {props.iconStart && <span>{props.iconStart}</span>}
    <span>{props.label}</span>
    {props.iconEnd && <span>{props.iconEnd}</span>}
  </button>
);
