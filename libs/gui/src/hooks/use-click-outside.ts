import { $, useOnDocument } from '@builder.io/qwik';
import type { QRL, Signal } from '@builder.io/qwik';

export const useClickOutside = (
  ref: Signal<HTMLElement | undefined>,
  onClickOut: QRL<() => void>,
) =>
  useOnDocument(
    'click',
    $((event) => {
      if (!ref.value) {
        return;
      }
      const target = event.target as HTMLElement;
      if (!ref.value.contains(target)) {
        onClickOut();
      }
    }),
  );
