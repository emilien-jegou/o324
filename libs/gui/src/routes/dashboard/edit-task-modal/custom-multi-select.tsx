import { component$, useSignal } from '@builder.io/qwik';
import * as R from 'remeda';
import { MultiSelect } from '~/ui/common/multi-select';
import type { PropFunction, Signal } from '@builder.io/qwik';
import type { MultiSelectOption } from '~/ui/common/multi-select';

type GrowingMultiSelectProps = {
  name: string;
  error?: boolean;
  class?: string;
  'bind:value': Signal<string[]>;
  placeholder: string;
  onChange$?: PropFunction<(a: string[]) => void>;
  defaultOptions: MultiSelectOption[];
};

export const GrowingMultiSelect = component$((props: GrowingMultiSelectProps) => {
  const tagsOptions = useSignal<MultiSelectOption[]>(props.defaultOptions);
  const input = useSignal<string | undefined>(undefined);

  return (
    <MultiSelect
      name={props.name}
      error={props.error}
      bind:value={props['bind:value']}
      class={props.class}
      onSelect$={(v) => {
        tagsOptions.value = R.pipe(
          tagsOptions.value,
          R.concat([{ value: v, label: v } as MultiSelectOption]),
          R.uniqueBy((o) => o.value),
        );
        props.onChange$?.(props['bind:value'].value);
      }}
      onSearchInput$={(v) => {
        input.value = v || undefined;
      }}
      placeholder={props.placeholder}
      options={R.pipe(
        [
          input.value?.length ? { label: input.value, value: input.value } : undefined,
          ...tagsOptions.value,
        ],
        R.filter((x): x is MultiSelectOption => !!x),
        R.uniqueBy((o) => o.value),
      )}
    />
  );
});
