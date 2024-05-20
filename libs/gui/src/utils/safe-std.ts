import { failable } from './result';

export const safeJsonParse = <T = unknown>(value: string) => failable(() => JSON.parse(value) as T);
export const safeJsonStringify = (value: unknown) => failable(() => JSON.stringify(value));

export const safeAtob = (data: string) => failable(() => atob(data));

// array.indexOf but return null instead of 0 if no value is not found
export const indexOf = <T, K extends T>(array: T[], needle: K): number | null => {
  const idx = array.indexOf(needle);
  if (idx === -1) return null;
  return idx;
};

// array.findIndex but return null instead of 0 if no value is not found
export const findIndex = <T>(array: T[], needleFn: (v: T) => boolean): number | null => {
  const idx = array.findIndex(needleFn);
  if (idx === -1) return null;
  return idx;
};
