import { useTask$ } from '@builder.io/qwik';
import { updateTrayIcon } from '~/api';
import type { Signal } from '@builder.io/qwik';

// control the whether the tray icon uses the active or inactive variant
export const useTrayIconUpdater = (currentTask: Signal<unknown | undefined>) => {
  useTask$(({ track }) => {
    // eslint-disable-next-line qwik/valid-lexical-scope
    track(() => !currentTask.value);
    // eslint-disable-next-line qwik/valid-lexical-scope
    updateTrayIcon(!!currentTask.value);
  });
};
