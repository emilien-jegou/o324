import { $ } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { Portal } from '~/provider/portal-context';
import { Dialog } from '../dialog';
import type { JSXChildren, Signal } from '@builder.io/qwik';

import './styles.css';

type ModalProps = {
  contentClass?: string;
  'bind:show': Signal<boolean>;
  children: JSXChildren;
};

// TODO: currently the modal is being closed onKeyDown$, we want to be closed on click only
// TODO: AppBlur is kinda buggy
export const Modal = ({ 'bind:show': show, children, contentClass }: ModalProps) => (
  <Portal
    name="modal"
    open={show.value}
    render$={$(() => (
      <Dialog
        role="dialog"
        class="z-[60] absolute w-screen h-screen top-0 left-0 bg-black/60"
        onMouseDown$={$(() => {
          show.value = false;
        })}
      >
        <div class="flex w-full h-full items-center justify-center">
          <div
            onMouseDown$={(e) => {
              e.stopPropagation();
            }}
            class={twMerge(
              'modal-animation w-full max-w-2xl gap-4 border border-space-700 bg-space-800 p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg bg-space-900 text-white',
              contentClass,
            )}
          >
            {
              // eslint-disable-next-line qwik/valid-lexical-scope
              children
            }
          </div>
        </div>
      </Dialog>
    ))}
  />
);
