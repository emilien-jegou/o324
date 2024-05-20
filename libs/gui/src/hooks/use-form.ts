import { $, useStore, useSignal, useVisibleTask$ } from '@builder.io/qwik';
import * as R from 'remeda';
import {
  type GetKeysDeep,
  type GetValueDeep,
  setIndexedValue,
  removeIndexedValue,
} from '~/utils/deep-indexing';
import { chainSuccess } from '~/utils/result';
import { safeJsonParse, safeJsonStringify } from '~/utils/safe-std';
import { getStorage, type StorageType } from '~/utils/storage';
import type { QRL, Signal } from '@builder.io/qwik';
import type {
  FieldValues,
  FormErrors,
  MaybeFunction,
  PartialValues,
  ValidateForm,
} from '@modular-forms/qwik';
import type { DeepPartial } from 'ts-essentials';
import type { SafeParseReturnType, ZodType } from 'zod';

export type OnSuccess<FormValues> = QRL<(data: FormValues) => void>;

type PersistFormOptions<FormValues> = {
  // Default to session storage
  storageType: StorageType;
  ignoreKeys: GetKeysDeep<DeepPartial<FormValues>>[];
};

type FormOptions<FormValues extends Record<string, any>> = {
  defaultValue?: DeepPartial<FormValues>;
  validate: QRL<ValidateForm<FormValues>>;
  onSuccess$: OnSuccess<FormValues>;
  // inactive by default
  persistFormValues?: boolean | Partial<PersistFormOptions<FormValues>>;
};

export type UseFormRet<FormValues extends Record<string, any>> = {
  fields: DeepPartial<FormValues>;
  // This signal can be watched to know when a form value changed
  updateCount: Signal<number>;
  errors: FormErrors<FormValues>;
  hasError: Signal<boolean>;
  setFormValue$: QRL<
    <const K extends GetKeysDeep<DeepPartial<FormValues>>>(
      deepKey: K,
      value: GetValueDeep<DeepPartial<FormValues>, K>,
    ) => void
  >;
  formSubmit$: QRL<() => void>;
};

export const useForm = <FormValues extends Record<string, any>>(
  formUniqKey: string,
  { defaultValue, validate, onSuccess$, persistFormValues }: FormOptions<FormValues>,
): UseFormRet<FormValues> => {
  const updateCount = useSignal(0);
  const initialized = useSignal(false);
  const submitted = useSignal(false);
  const hasError = useSignal<boolean>(false);
  const fields = useStore<DeepPartial<FormValues>>(
    () => defaultValue ?? ({} as DeepPartial<FormValues>),
  );
  const errors = useStore<FormErrors<FormValues>>({});

  const persistenceOptions: PersistFormOptions<FormValues> = {
    ...({
      storageType: 'session',
      ignoreKeys: [],
    } satisfies PersistFormOptions<FormValues>),
    ...(typeof persistFormValues === 'boolean' ? {} : persistFormValues),
  };

  const getFormStorage = $(() =>
    getStorage<DeepPartial<FormValues>>(persistenceOptions.storageType, 'useForm--' + formUniqKey),
  );

  // eslint-disable-next-line qwik/no-use-visible-task
  useVisibleTask$(async ({ track }) => {
    if (!persistFormValues) return;
    track(() => updateCount.value);
    // TODO: this should be debounced for performance reasons
    const formStorage = await getFormStorage();

    // On first render, if the value is already in the persistent storage,
    // load it into the given signal.
    if (!initialized.value) {
      initialized.value = true;
      const current = formStorage.get();
      if (current.kind === 'none') return;
      const newKeys = new Set(Object.keys(current.value));
      for (const key of Object.keys(fields)) {
        if (!newKeys.has(key as string)) {
          delete (fields as any)[key];
        }
      }

      for (const key of [...newKeys]) {
        fields[key as keyof DeepPartial<FormValues>] =
          current.value[key as keyof DeepPartial<FormValues>];
      }
      return;
    }

    // If the value of the signal changed, re-save the persistent storage with
    // the new value.
    // TODO: use structuredClone
    //const data = structuredClone({ ...fields });
    const data = R.pipe(safeJsonStringify(fields), chainSuccess(safeJsonParse));
    if (data.kind === 'success') {
      const value = data.value as any;
      persistenceOptions.ignoreKeys.forEach((key) => removeIndexedValue(value, key));
      formStorage.set(value);
    }
  });

  // eslint-disable-next-line qwik/no-use-visible-task
  useVisibleTask$(async ({ track }) => {
    track(() => submitted.value);
    track(() => updateCount.value);
    if (submitted.value === false) return;
    // TODO: speed this up
    const res = await validate(fields as PartialValues<FormValues>);
    hasError.value = Object.keys(res).length !== 0;
    const left = new Set(Object.keys(res));
    Object.keys(errors)
      .filter((k) => !left.has(k))
      .forEach((k) => {
        // Reset old errors
        (errors as any)[k] = undefined;
      });

    Object.entries(res).forEach(([k, v]) => {
      (errors as any)[k] = v;
    });
  });

  const setFormValue$ = $(
    <const K extends GetKeysDeep<DeepPartial<FormValues>>>(
      deepKey: K,
      value: GetValueDeep<DeepPartial<FormValues>, K>,
    ) => {
      updateCount.value++;
      setIndexedValue(fields, deepKey, value);
    },
  );

  const formSubmit$ = $(async () => {
    if (hasError.value) return;
    submitted.value = true;
    const res = await validate(fields as PartialValues<FormValues>);

    hasError.value = Object.keys(res).length !== 0;
    if (!hasError.value) {
      await onSuccess$(fields as FormValues);

      // Clear the form storage on success
      if (persistFormValues) {
        const formStorage = await getFormStorage();
        formStorage.remove();
      }
      return;
    }

    const left = new Set(Object.keys(res));
    Object.keys(errors)
      .filter((k) => !left.has(k))
      .forEach((k) => {
        // Reset old errors
        (errors as any)[k] = undefined;
      });

    Object.entries(res).forEach(([k, v]) => {
      (errors as any)[k] = v;
    });
  });

  return { hasError, updateCount, errors, fields, formSubmit$, setFormValue$ };
};

export async function getParsedZodSchema<Schema, Value>(
  schema: QRL<MaybeFunction<ZodType<any, any, Schema>>>,
  value: Value,
): Promise<SafeParseReturnType<Schema, any>> {
  const zodSchema = await schema.resolve();
  return (typeof zodSchema === 'function' ? zodSchema() : zodSchema).safeParse(value);
}

export function validate<TFieldValues extends FieldValues>(
  schema: QRL<ZodType<any, any, TFieldValues>>,
): QRL<ValidateForm<TFieldValues>> {
  return $(async (values: PartialValues<TFieldValues>) => {
    const result = await getParsedZodSchema(schema, values);
    // eslint-disable-next-line qwik/valid-lexical-scope
    //const result = schema.safeParse(values);
    return result.success
      ? ({} as FormErrors<TFieldValues>)
      : (result.error.issues.reduce<any>((errors, error) => {
          const path = error.path.join('.');
          if (!errors[path]) {
            errors[path] = error.message;
          }
          return errors;
        }, {}) as FormErrors<TFieldValues>);
  });
}
