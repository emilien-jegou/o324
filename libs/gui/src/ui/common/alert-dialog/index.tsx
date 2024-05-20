import { Button } from '../button';
import { Modal } from '../modal';
import type { PropFunction, Signal } from '@builder.io/qwik';

type AlertDialogProps = {
  title: string;
  description: string;
  onContinue$?: PropFunction<() => void>;
  'bind:show': Signal<boolean>;
};

export const AlertDialog = (props: AlertDialogProps) => (
  <Modal bind:show={props['bind:show']}>
    <div class="flex flex-col space-y-2 text-center sm:text-left">
      <h2 id="radix-:rg:" class="text-lg font-semibold text-ellipsis overflow-hidden">
        {props.title}
      </h2>
      <p id="radix-:rh:" class="text-sm text-muted-foreground text-ellipsis overflow-hidden">
        {props.description}
      </p>
    </div>
    <div class="flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2">
      <Button
        variant="outlined"
        onClick$={() => {
          props['bind:show'].value = false;
        }}
        class="mt-2 sm:mt-0"
      >
        Cancel
      </Button>
      <Button onClick$={props.onContinue$} class="hover:bg-red-600">
        Continue
      </Button>
    </div>
  </Modal>
);
