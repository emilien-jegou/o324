import { some, type Option, none } from './option';
import type { Prettify } from 'ts-essentials';

export type Result<T = unknown, E = unknown> = Success<T> | Failure<E>;

export interface Success<T> {
  kind: 'success';
  value: T;
}

export interface Failure<E> {
  kind: 'failure';
  error: E;
}

export const success = <T>(value: T): Success<T> => {
  return {
    kind: 'success',
    value: value,
  };
};

export const failure = <E>(error: E): Failure<E> => {
  return {
    kind: 'failure',
    error: error,
  };
};

// run code in a 'failable' context where thrown exception are converted to a result error
export const failable = <const T>(cb: () => T): Result<T, unknown> => {
  try {
    return success(cb());
  } catch (e: unknown) {
    return failure<unknown>(e);
  }
};

export const failableAsync = async <const T>(cb: () => Promise<T>): Promise<Result<T, unknown>> => {
  try {
    return success(await cb());
  } catch (e: unknown) {
    return failure<unknown>(e);
  }
};

// transform a throwable function into a function that return a result type
export const safeify = <const CB extends (...args: any) => any>(
  cb: CB,
): ((...args: Parameters<CB>) => Result<Prettify<ReturnType<CB>>, unknown>) => {
  return (...args: any) => failable(() => cb(args));
};

export const safeifyAsync = <const Args extends any[], const Ret>(
  cb: (...args: Args) => Promise<Ret>,
): ((...args: Args) => Promise<Result<Ret, unknown>>) => {
  return (...args: any) => failableAsync(() => cb(...args));
};

export const ok = <T>(result: Result<T, unknown>): Option<T> => {
  if (result.kind == 'success') return some(result.value);
  else return none();
};

// convert throwable function into a function that return a result
export const unwrap_or =
  <const V>(or: V) =>
  <T>(result: Option<T> | Result<T, unknown>): T | V => {
    if (result.kind === 'success' || result.kind === 'some') {
      return result.value;
    } else {
      return or;
    }
  };

export const mapSuccess =
  <T, V, E>(cb: (d: T) => V) =>
  (o: Result<T, E>) => {
    if (o.kind == 'success') {
      return success(cb(o.value));
    } else {
      return o;
    }
  };

export const chainSuccess =
  <T, V, E, R>(cb: (d: T) => Result<V, E | R>) =>
  (o: Result<T, E>) => {
    if (o.kind == 'success') {
      return cb(o.value);
    } else {
      return o;
    }
  };
