import { twMerge } from 'tailwind-merge';
import { Field } from '../field';
import { MultiSelect, type MultiSelectProps } from '../multi-select';
import type { JSXChildren } from '@builder.io/qwik';
import type { Classes } from '~/utils/types';

export type MultiSelectFieldProps = {
  classes?: Classes<'root' | 'select'>;
  error?: string;
  helperText?: string;
  info?: JSXChildren;
  label: string;
  required?: boolean;
} & Omit<MultiSelectProps, 'error'>;

export const MultiSelectField = ({
  classes,
  info,
  label,
  required,
  error,
  helperText,
  ...props
}: MultiSelectFieldProps) => (
  <Field
    class={classes?.root}
    info={info}
    label={label}
    required={required}
    error={error}
    helperText={helperText}
  >
    <MultiSelect class={twMerge('w-full', classes?.select)} error={!!error} {...props} />
  </Field>
);
