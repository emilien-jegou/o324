import { component$ } from '@builder.io/qwik';
import { cn } from '~/utils/cn';
import { Field } from '../field';
import { Select } from '../select';
import type { SelectProps } from '../select';
import type { JSXChildren } from '@builder.io/qwik';
import type { Classes } from '~/utils/types';

export type SelectFieldProps = {
  classes?: Classes<'root' | 'select'>;
  error?: string;
  helperText?: string;
  info?: JSXChildren;
  label: string;
  required?: boolean;
  disabled?: boolean;
} & Omit<SelectProps, 'error'>;

export const SelectField = component$(
  ({ classes, info, label, required, error, helperText, ...props }: SelectFieldProps) => (
    <Field
      class={classes?.root}
      info={info}
      label={label}
      required={required}
      error={error}
      helperText={helperText}
    >
      <Select error={Boolean(error)} class={cn('w-full', classes?.select)} {...props} />
    </Field>
  ),
);
