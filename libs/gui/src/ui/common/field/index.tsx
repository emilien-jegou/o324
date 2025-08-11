import { cn } from '~/utils/cn';
import { focusNext } from '~/utils/focus';
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
  <div
    onKeyDown$={(event) => {
      if (event.key === 'Enter') {
        focusNext();
      }
    }}
    class={className}
  >
    <Label
      info={info}
      classes={{ root: 'mb-2' }}
      text={label}
      required={required}
      tooltipPosition="bottom-left"
    />
    {children}
    <p class={cn('mt-1 text-sm text-space-400 select-none', error && 'text-red-500')}>
      {error || helperText}&nbsp;
    </p>
  </div>
);
