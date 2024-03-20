import type { JSXChildren } from '@builder.io/qwik';

type FieldProps = {
  position: number;
  children: JSXChildren;
};

export const Field = ({ position, children }: FieldProps) => (
  <div style={{ transform: `translateX(${position * 100}%)` }} class="absolute w-full">
    {children}
  </div>
);
