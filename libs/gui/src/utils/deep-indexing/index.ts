import { type Option, none, some } from '../option';

// Remove traling character of type C, e.g.
// RemoveTrailing<'hello', 'o'> = 'hell'
// RemoveTrailing<'hello', '-'> = 'hello'
type RemoveTrailing<S extends string, C extends string> = S extends `${infer Prefix}${C}`
  ? RemoveTrailing<Prefix, C>
  : S;

// Return deep keys of an object T, e.g.:
// GetKeysDeep<{ a: { b: 0, c: 0 } }>  = 'a' | 'a.b' | 'a.c'
export type GetKeysDeep<T, Separator extends string = '.'> =
  T extends Record<string, unknown>
    ? RemoveTrailing<
        {
          [K in keyof T]-?: K extends string ? `${K}${Separator}${GetKeysDeep<T[K]>}` | K : never;
        }[keyof T],
        Separator
      >
    : '';

// Given an object T and a key path, return the value type, e.g.:
// GetValueDeep<{ a: { b: boolean } }, 'a.b'> = boolean
export type GetValueDeep<
  T,
  K extends GetKeysDeep<T>,
  Separator extends string = '.',
> = K extends `${infer Key}${Separator}${infer Rest}`
  ? Key extends keyof T
    ? Rest extends GetKeysDeep<T[Key]>
      ? GetValueDeep<T[Key], Rest>
      : never
    : never
  : T extends undefined
    ? never
    : K extends keyof T
      ? T[K]
      : never;

export const getIndexedValue = <T, K extends GetKeysDeep<T>>(
  obj: T,
  needle: K,
): Option<GetValueDeep<T, K>> => {
  const keys = needle.split('.');
  let result: any = obj;
  for (const k of keys) {
    if (result && typeof result === 'object' && k in result) {
      result = result[k];
    } else {
      return none();
    }
  }
  return some(result as GetValueDeep<T, K>);
};

export const setIndexedValue = <T, K extends GetKeysDeep<T>>(
  obj: T,
  needle: K,
  value: GetValueDeep<T, K>,
) => {
  const keys = needle.split('.');
  let currentObj: any = obj;

  for (let i = 0; i < keys.length - 1; i++) {
    const k = keys[i];
    if (typeof currentObj[k] !== 'object' || !currentObj[k]) {
      currentObj[k] = {};
    }
    currentObj = currentObj[k];
  }

  currentObj[keys[keys.length - 1]] = value;
};

export const removeIndexedValue = <T, K extends GetKeysDeep<T>>(obj: T, needle: K) => {
  const keys = needle.split('.');
  let currentObj: any = obj;

  for (let i = 0; i < keys.length - 1; i++) {
    const k = keys[i];
    if (typeof currentObj[k] !== 'object' || !currentObj[k]) {
      currentObj[k] = {};
    }
    currentObj = currentObj[k];
  }

  delete currentObj[keys[keys.length - 1]];
};
