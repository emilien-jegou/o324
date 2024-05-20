import { describe, expect, test } from 'vitest';
import { getIndexedValue, setIndexedValue } from '.';
import { none, some } from '../option';

describe.concurrent('getIndexedValue', () => {
  test('retrieving a non indexed value', () => {
    const value = getIndexedValue({ a: { b: 42 } }, 'a');

    expect(value).toStrictEqual(some({ b: 42 }));
  });

  test('retrieving an indexed value 1 level deep', () => {
    const value = getIndexedValue({ a: { b: 42 } }, 'a.b');

    expect(value).toStrictEqual(some(42));
  });

  test('retrieving a indexed value 60 level deep on recursive object', () => {
    // Create a recursive object
    const a: any = { b: 42 };
    a.a = a;

    // Create a string for indexing in the recursive object
    const index = Array.from({ length: 60 }, () => 'a').join('.') + '.b';

    const value = getIndexedValue(a, index);

    expect(value).toStrictEqual(some(42));
  });

  test('retrieving a missing value', () => {
    const value = getIndexedValue({ a: 42 }, 'v' as any);

    expect(value).toStrictEqual(none());
  });

  test('retrieving a missing value, indexed key', () => {
    const value = getIndexedValue({ a: 42 }, 'v.b.c.a' as any);

    expect(value).toStrictEqual(none());
  });

  test('passing in empty string', () => {
    const value = getIndexedValue({ a: { b: 42 } }, '' as any);

    expect(value).toStrictEqual(none());
  });

  test('passing indexed empty string', () => {
    const value = getIndexedValue({ ['']: { ['']: 42 } }, '.' as any);

    expect(value).toStrictEqual(some(42));
  });
});

describe.concurrent('setIndexedValue', () => {
  test('setting an indexed value 4 level deep', () => {
    const obj: any = {};
    setIndexedValue(obj, 'a.b.c.d', 42);

    expect(obj).toStrictEqual({ a: { b: { c: { d: 42 } } } });
  });

  test('overwriting object indexed value', () => {
    const obj: any = { a: { b: { c: 42 }, y: 0 }, z: 0 };

    setIndexedValue(obj, 'a.b', 42);

    expect(obj).toStrictEqual({ a: { b: 42, y: 0 }, z: 0 });
  });

  test('passing in empty string', () => {
    const obj: any = {};

    setIndexedValue(obj, '', 42);

    expect(obj).toStrictEqual({ ['']: 42 });
  });

  test('passing indexed empty string', () => {
    const obj: any = {};

    setIndexedValue(obj, '.', 42);

    expect(obj).toStrictEqual({ ['']: { ['']: 42 } });
  });
});
