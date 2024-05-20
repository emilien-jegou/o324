import { useVisibleTask$ } from '@builder.io/qwik';
import { createFocusTrap } from 'focus-trap';
import { failable } from '~/utils/result';
import type { Signal } from '@builder.io/qwik';

export const useFocusTrap = (ref: Signal<HTMLElement | undefined>) =>
  useVisibleTask$(() => {
    if (!ref.value) return;
    try {
      const trap = createFocusTrap(ref.value, { escapeDeactivates: false });

      trap.activate();
      return () => {
        failable(() => trap.deactivate());
      };
    } catch {
      // Activating the focus trap throws if no tabbable elements are inside the container.
      // If this is the case we are fine with not activating the focus trap.
      // That's why we ignore the thrown error.
      return () => {};
    }
  });
