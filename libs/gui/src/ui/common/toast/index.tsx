import { $, useSignal, component$, useTask$ } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { DismissIcon } from '~/ui/icons/dismiss';
import type { AddPortal } from '~/provider/portal-context';

type ToastProps = {
  status: 'success' | 'error';
  title: string;
  description: string;
};

// TODO: fix this toast component, it's a bit buggy
export const Toast = component$((props: ToastProps) => {
  const isOpen = useSignal(true);
  const isRemoved = useSignal(false);
  const timerId = useSignal<number | null>();

  const restartCloseTimer$ = $(() => {
    if (timerId.value !== null) return;
    timerId.value = Number(
      setTimeout(() => {
        isOpen.value = false;
      }, 8000),
    );
  });

  useTask$(() => {
    restartCloseTimer$();
  });

  useTask$(({ track }) => {
    track(() => isOpen.value);
    if (isOpen.value === true) {
      isRemoved.value = false;
      return () => {};
    } else {
      const timerId = setTimeout(() => {
        isRemoved.value = true;
      }, 80);

      return () => clearTimeout(timerId);
    }
  });

  return (
    <>
      {isRemoved.value == false && (
        <li
          role="status"
          aria-live="off"
          aria-atomic="true"
          onMouseOver$={() => {
            if (timerId.value !== null) {
              clearTimeout(timerId.value);
              timerId.value = null;
            }
          }}
          onMouseOut$={() => restartCloseTimer$()}
          tabIndex={0}
          data-state={isOpen.value ? 'open' : 'closed'}
          data-swipe-direction="right"
          class={twMerge(
            'pointer-events-auto relative flex w-full items-center justify-between space-x-2 overflow-hidden rounded-md border p-4 pr-6 shadow-lg transition-all data-[swipe=cancel]:translate-x-0 data-[swipe=end]:translate-x-[var(--radix-toast-swipe-end-x)] data-[swipe=move]:translate-x-[var(--radix-toast-swipe-move-x)] data-[swipe=move]:transition-none data-[state=open]:animate-in data-[state=closed]:animate-out data-[swipe=end]:animate-out data-[state=closed]:fade-out-80 data-[state=closed]:slide-out-to-right-full data-[state=open]:slide-in-from-top-full data-[state=open]:sm:slide-in-from-bottom-full group border-destructive text-contrast',
            props.status === 'error' && 'bg-error',
            props.status === 'success' && 'bg-success',
          )}
          data-radix-collection-item=""
          style="user-select: none; touch-action: none;"
        >
          <div class="grid gap-1">
            <div class="text-sm font-semibold [&amp;+div]:text-xs">{props.title}</div>
            <div class="text-sm opacity-90">{props.description}</div>
          </div>
          <button
            type="button"
            class="absolute right-1 top-1 rounded-md p-1 text-contrast opacity-70 transition-opacity focus:opacity-100 focus:outline-none focus:ring-1"
            onClick$={() => {
              isOpen.value = false;
            }}
          >
            <DismissIcon />
          </button>
        </li>
      )}
    </>
  );
});

export const sendToast = (
  portal: AddPortal,
  options: Pick<ToastProps, 'title' | 'description' | 'status'>,
) => {
  portal(
    'toast',
    (
      <ol
        tabIndex={-1}
        class="fixed top-0 z-[1000000] flex max-h-screen w-full flex-col-reverse p-4 sm:bottom-0 sm:right-0 sm:top-auto sm:flex-col md:max-w-[420px]"
      >
        <Toast title={options.title} description={options.description} status={options.status} />
      </ol>
    ) as any,
  );
};
