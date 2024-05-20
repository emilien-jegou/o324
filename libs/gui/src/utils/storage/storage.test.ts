import { describe, expect, test } from 'vitest';
import { safeStorage } from '.';
import { none, some } from '../option';
import type { RawStorage } from '.';

const getTestRawStorage = (): RawStorage & { store: Partial<Record<string, string>> } => {
  const store: Partial<Record<string, string>> = {};

  return {
    store,
    getItem(key: string): string | null {
      return store[key] ?? null;
    },
    setItem(key: string, value: string) {
      store[key] = value;
    },
    removeItem(key: string) {
      delete store[key];
    },
  };
};

describe.concurrent('safeStorage', () => {
  test('get an empty element', () => {
    const rawStorage = getTestRawStorage();
    const storage = safeStorage('test', rawStorage);

    const result = storage.get();

    expect(result).toStrictEqual(none());
  });

  test('get a non empty element', () => {
    const rawStorage = getTestRawStorage();
    rawStorage.store['test'] = '{ "a": 32 }';
    const storage = safeStorage('test', rawStorage);

    const result = storage.get();

    expect(result).toStrictEqual(some({ a: 32 }));
  });

  test('get a non parseable element', () => {
    const rawStorage = getTestRawStorage();
    rawStorage.store['test'] = '{';
    const storage = safeStorage('test', rawStorage);

    const result = storage.get();

    expect(result).toStrictEqual(none());
  });

  test('setting an element', () => {
    const rawStorage = getTestRawStorage();
    const storage = safeStorage('test', rawStorage);

    const result = storage.set({ a: 32 });

    expect(result.kind).toStrictEqual('success');
    expect(rawStorage.store).toStrictEqual({ test: '{"a":32}' });
  });

  test('setting an erroneous object', () => {
    const rawStorage = getTestRawStorage();
    const storage = safeStorage('test', rawStorage);

    // Creating a recursive object (can't be stringified)
    const recursiveObject: any = { b: 32 };
    recursiveObject.a = recursiveObject;

    const result = storage.set(recursiveObject);

    expect(result.kind).toStrictEqual('failure');
    expect(rawStorage.store).toStrictEqual({});
  });

  test('removing an element', () => {
    const rawStorage = getTestRawStorage();
    rawStorage.store['test'] = '{"a":42}';
    const storage = safeStorage('test', rawStorage);

    const result = storage.remove();

    expect(result.kind).toStrictEqual('success');
    expect(rawStorage.store).toStrictEqual({});
  });

  test('removing an empty element', () => {
    const rawStorage = getTestRawStorage();
    const storage = safeStorage('test', rawStorage);

    const result = storage.remove();

    expect(result.kind).toStrictEqual('success');
    expect(rawStorage.store).toStrictEqual({});
  });
});
