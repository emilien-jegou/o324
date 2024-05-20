import { $, component$, useContext, useVisibleTask$ } from '@builder.io/qwik';
import { FocusCycleContext } from '../focus-cycle-controller';
import type { JSXOutput, QRL } from '@builder.io/qwik';

type FocusCycleNodeProps = {
  position: number;
  render$: QRL<(focused: boolean, focus: QRL<() => void>) => JSXOutput>;
};

export const FocusCycleNode = component$((props: FocusCycleNodeProps) => {
  const focusCycleContext = useContext(FocusCycleContext);
  //const focused = useSignal<HTMLElement>();

  // eslint-disable-next-line qwik/no-use-visible-task
  useVisibleTask$(() => {
    //track(() => ref.value);
    //if (!ref.value) return;
    return focusCycleContext.register$();
  });

  return (
    <>
      {props.render$(
        focusCycleContext.currentPosition.value === props.position,
        $(() => {
          focusCycleContext.focus$(props.position);
        }),
      )}
    </>
  );
});
