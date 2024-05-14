import { component$, Slot } from '@builder.io/qwik';
import { Modal as HModal, ModalContent } from '@qwik-ui/headless';
import { twMerge } from 'tailwind-merge';
import type { ModalProps as HModalProps } from '@qwik-ui/headless';

import './styles.css';

type ModalProps = {
  disableBackground?: boolean;
} & HModalProps;

export const Modal = component$((props: ModalProps) => (
  <HModal
    {...props}
    class={twMerge(
      'modal-animation bg-transparent',
      !props.disableBackground &&
        'bg-default w-full max-w-lg border bg-background p-6 shadow-lg sm:rounded-lg sm:max-w-[640px]',
    )}
  >
    <ModalContent>
      <Slot />
    </ModalContent>
  </HModal>
));
