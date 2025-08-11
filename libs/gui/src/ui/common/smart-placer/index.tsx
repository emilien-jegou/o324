import { component$ } from '@builder.io/qwik';
import type { Prettify } from 'ts-essentials';

type YAxis = 'top' | 'bottom';
type XAxis = 'left' | 'right';

export type SmartPlacerPosition = Prettify<XAxis | YAxis | `${YAxis}-${XAxis}`>;

type SmartPlacerProps = {
  defaultPosition: SmartPlacerPosition;
};

export const SmartPlacer = component$((props: SmartPlacerProps) => {
  return <div></div>;
});
