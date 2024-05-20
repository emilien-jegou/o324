import { twMerge } from 'tailwind-merge';
import { Field } from '../field';
import { Input } from '../input';
import type { InputProps } from '../input';
import type { JSXChildren } from '@builder.io/qwik';
import type { Classes } from '~/utils/types';

export type InputFieldProps = {
  classes?: Classes<'root' | 'input'>;
  error?: string;
  helperText?: string;
  info?: JSXChildren;
  label: string;
  required?: boolean;
} & Omit<InputProps, 'error'>;

export const InputField = ({
  classes,
  info,
  label,
  required,
  error,
  helperText,
  ...props
}: InputFieldProps) => (
  <Field
    class={classes?.root}
    info={info}
    label={label}
    required={required}
    error={error}
    helperText={helperText}
  >
    <Input error={Boolean(error)} class={twMerge('w-full', classes?.input)} {...props} />
  </Field>
);
