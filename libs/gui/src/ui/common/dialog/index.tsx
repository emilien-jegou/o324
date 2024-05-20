import { Slot, component$, useSignal } from '@builder.io/qwik';
import { useFocusTrap } from '~/hooks/use-focus-trap';
import type { HTMLAttributes } from '@builder.io/qwik';

type DialogProps = Omit<HTMLAttributes<HTMLDialogElement>, 'ref'>;

export const Dialog = component$((props: DialogProps) => {
  const dialogRef = useSignal<HTMLDialogElement | undefined>(undefined);

  useFocusTrap(dialogRef);

  return (
    <dialog ref={dialogRef} open={true} {...props}>
      <Slot />
    </dialog>
  );
});
