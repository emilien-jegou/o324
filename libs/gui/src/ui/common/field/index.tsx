import { twMerge } from 'tailwind-merge';
import { Label } from '../label';
import type { JSXChildren } from '@builder.io/qwik';

export type FieldProps = {
  class?: string;
  error?: string;
  helperText?: string;
  info?: JSXChildren;
  label: string;
  required?: boolean;
  children: JSXChildren;
};

export const Field = ({
  class: className,
  info,
  label,
  required,
  error,
  helperText,
  children,
}: FieldProps) => (
  <div class={className}>
    <Label info={info} classes={{ root: 'mb-2' }} text={label} required={required} />
    {children}
    <p class={twMerge('mt-1 text-xs text-white select-none', error && 'text-red-500')}>
      {error || helperText}&nbsp;
    </p>
  </div>
);
