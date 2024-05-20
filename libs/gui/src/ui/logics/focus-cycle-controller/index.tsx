import {
  $,
  Slot,
  component$,
  createContextId,
  useContextProvider,
  useSignal,
} from '@builder.io/qwik';
import type { HTMLAttributes, PropFunction, QRL, Signal } from '@builder.io/qwik';

type FocusCycleContextData = {
  currentPosition: Signal<number>;
  length: Signal<number>;
  register$: QRL<() => () => void>;
  focus$: QRL<(position: number) => void>;
};

export const FocusCycleContext = createContextId<FocusCycleContextData>('FocusCycleContext');

type FocusCycleControllerProps = {
  'bind:position': Signal<number>;
  onKeyDown$?: PropFunction<(event: KeyboardEvent) => void>;
  disabled?: boolean;
} & Omit<HTMLAttributes<HTMLDivElement>, 'onKeyDown$'>;

export const FocusCycleController = component$(
  ({
    'bind:position': currentPosition,
    disabled,
    onKeyDown$,
    ...props
  }: FocusCycleControllerProps) => {
    const length = useSignal(0);
    const contextData = {
      currentPosition,
      length,
      focus$: $((position: number) => {
        currentPosition.value = position;
      }),
      register$: $(() => {
        length.value += 1;
        return () => {
          currentPosition.value = currentPosition.value % length.value;
          if (currentPosition.value == length.value) {
            currentPosition.value = 0;
          }
          length.value -= 1;
        };
      }),
    };

    useContextProvider(FocusCycleContext, contextData);

    return (
      <div
        onKeyDown$={(event: KeyboardEvent) => {
          if (disabled === true) return;
          if (event.key === 'Tab' || event.key === 'ArrowDown') {
            currentPosition.value = (currentPosition.value + 1) % length.value || 0;
            event.preventDefault(); // Prevent the default tab behavior
          } else if (event.key === 'ArrowUp') {
            currentPosition.value = (length.value + currentPosition.value - 1) % length.value || 0;
            event.preventDefault(); // Prevent the default tab behavior
          }
          onKeyDown$?.(event);
        }}
        {...props}
      >
        <Slot />
      </div>
    );
  },
);
