export type ExtractWhen<T, V> = { [K in keyof T]-?: T[K] extends V ? K : never };

export type OnlyString<X> = X extends string ? X : never;
export type KeysMatching<T, V> = OnlyString<ExtractWhen<T, V>[keyof T]>;

export type Classes<T extends string> = Partial<Record<T, string>>;

export type IsArray<T> = T extends Array<any> ? true : false;
export type IsObject<T> = T extends object ? (false extends IsArray<T> ? true : false) : false;
