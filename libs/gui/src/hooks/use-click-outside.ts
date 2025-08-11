import { $, useOnDocument } from '@builder.io/qwik';
import type { QRL, Signal } from '@builder.io/qwik';

export const useClickOutside = (
  refs: Signal<HTMLElement | undefined>[],
  onClickOut: QRL<() => void>,
) =>
  useOnDocument(
    'click',
    $((event) => {
      const refsVals: HTMLElement[] = refs
        .map(({ value }) => value)
        .filter((x): x is HTMLElement => !!x);

      if (!refsVals.length) return;
      const target = event.target as HTMLElement;

      refsVals.map((val) => val.contains(target));
      for (const it of refsVals) {
        if (it.contains(target)) return;
      }
      onClickOut();
    }),
  );
