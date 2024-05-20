import { twMerge } from 'tailwind-merge';
import type { HTMLAttributes, JSXOutput } from '@builder.io/qwik';

type ButtonProps = {
  children: JSXOutput;
  class?: string;
  variant?: 'filled' | 'outlined';
  type?: HTMLButtonElement['type'];
} & HTMLAttributes<HTMLButtonElement>;

export const Button = ({
  type,
  variant = 'filled',
  children,
  class: className,
  ...props
}: ButtonProps) => (
  <button
    type={type ?? 'button'}
    class={twMerge(
      'inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 border px-4 py-2',
      variant === 'outlined' && 'border-space-600 bg-transparent shadow-sm hover:bg-space-600',
      variant === 'filled' &&
        'border-space-600 bg-space-1000 text-primary-foreground shadow hover:bg-space-300 drop-shadow-xl',
      className,
    )}
    {...props}
  >
    {children}
  </button>
);
