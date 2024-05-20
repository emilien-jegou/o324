import { pipe } from 'remeda';
import { match } from 'ts-pattern';
import { chainSome, fromNullable, none, type Option } from '~/utils/option';
import { chainSuccess, failable, ok, unwrap_or } from '~/utils/result';
import { safeJsonParse, safeJsonStringify } from '~/utils/safe-std';
import type { Result } from '~/utils/result';

export type StorageType = 'local' | 'session';

export type Storage<T = unknown> = {
  get: () => Option<T>;
  set: (value: T) => Result;
  remove: () => Result;
};

export type RawStorage = {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
  removeItem: (key: string) => void;
};

export const safeStorage = <T>(storageKey: string, rawStorage: RawStorage): Storage<T> => ({
  get() {
    return pipe(
      failable(() => fromNullable(rawStorage.getItem(storageKey) as string | null)),
      unwrap_or(none()),
      chainSome((d) => ok(safeJsonParse<T>(d))),
    );
  },
  set(value: T) {
    return pipe(
      value,
      safeJsonStringify,
      chainSuccess((data: string) => failable(() => void rawStorage.setItem(storageKey, data))),
    );
  },
  remove() {
    return failable(() => void rawStorage.removeItem(storageKey));
  },
});

export const getStorage = <T>(storageType: StorageType, storageKey: string): Storage<T> =>
  safeStorage<T>(
    storageKey,
    match(storageType)
      .with('local', () => window.localStorage)
      .with('session', () => window.sessionStorage)
      .exhaustive(),
  );
