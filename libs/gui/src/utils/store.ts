import { Store } from '@tauri-apps/plugin-store';

type AppStore<T> = {
  get(): Promise<T>;
  set(value: T): Promise<void>;
  has(): Promise<boolean>;
  delete(): Promise<boolean>;
  onKeyChange(listener: (value: T | null) => void): Promise<() => void>;
};

const globalStore = new Store('o324-store.bin');

export const getStore = <T>(key: string, defaultValue: T): AppStore<T> => {
  return {
    get: async (): Promise<T> => {
      const item = await globalStore.get<T>(key);
      return item ?? defaultValue;
    },
    set: (value: T): Promise<void> => globalStore.set(key, value),
    has: (): Promise<boolean> => globalStore.has(key),
    delete: (): Promise<boolean> => globalStore.delete(key),
    onKeyChange: (listener: (value: T) => void): Promise<() => void> =>
      globalStore.onKeyChange(key, (v: T | null) => {
        listener(v ?? defaultValue);
      }),
  };
};

type KeyedStore<T> = {
  at(subkey: string): AppStore<T>;
  getAll(): Promise<Record<string, T>>;
};

export const getKeyedStore = <T>(key: string, defaultValue: T): KeyedStore<T> => ({
  at: (subkey: string): AppStore<T> => getStore([key, subkey].join('.'), defaultValue),
  getAll: async (): Promise<Record<string, T>> => {
    const entries = await globalStore.entries();
    const keyStart = `${key}.`;
    const entriesFiltered = entries
      .filter(([k]) => k.startsWith(keyStart))
      .map(([k, v]) => [k.slice(keyStart.length), v]);

    return Object.fromEntries(entriesFiltered);
  },
});
