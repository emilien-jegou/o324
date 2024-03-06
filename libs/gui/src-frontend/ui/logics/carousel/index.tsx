import { Slot, component$, useSignal, useVisibleTask$ } from '@builder.io/qwik';
import { Moveable } from '../moveable';
import type { PropFunction } from '@builder.io/qwik';

type CarouselProps = {
  fieldQuantity: number;
  position: number;
  onPositionChange$: PropFunction<(newPosition: number) => void>;
};

export const Carousel = component$(
  ({ position, onPositionChange$, fieldQuantity }: CarouselProps) => {
    const innerRef = useSignal<HTMLDivElement>();

    // eslint-disable-next-line qwik/no-use-visible-task
    useVisibleTask$(({ track }) => {
      track(() => position);

      if (innerRef.value === undefined) return;
      const selectedIdx = position > fieldQuantity - 1 ? fieldQuantity - 1 : position;

      innerRef.value!.style.left = `-${selectedIdx * 100}%`;
    });

    return (
      <div class="relative w-full">
        <div ref={innerRef} class="transition-all duration-[400ms] ease-out absolute w-full">
          <Moveable
            axis="x"
            onMoveEnd$={(data) => {
              if (!innerRef.value) return;
              // Take into account existing translate on the element to calculate initial position.
              const style = window.getComputedStyle(innerRef.value);
              const translateX = new DOMMatrix(style.transform).m41;

              // Readjust element position
              innerRef.value.style.transform = `translateX(${
                translateX - data.translate.x + data.initialTranslate.x
              }px)`;
              console.info(translateX, data.translate.x, data.initialTranslate.x);

              const rect = innerRef.value.getBoundingClientRect();

              // Coefficient of movement of a field, e.g. 0.2 => the item moved 20% to the right
              const coefAmountMoved = (data.translate.x - data.initialTranslate.x) / rect.width;

              const moveDurationMs = Number(data.end) - Number(data.start);

              let selectedIdx = position > fieldQuantity - 1 ? fieldQuantity : position;

              const absCoef = Math.abs(coefAmountMoved);

              if (
                (moveDurationMs < 400 && absCoef > 0.1) ||
                (moveDurationMs < 800 && absCoef > 0.2) ||
                (moveDurationMs < 1000 && absCoef > 0.3) ||
                absCoef > 0.4
              ) {
                selectedIdx += coefAmountMoved > 0 ? -1 : 1;
              }

              if (selectedIdx >= 0 && selectedIdx <= fieldQuantity) {
                onPositionChange$(selectedIdx);
              }
            }}
          >
            <div class="flex w-full">
              <Slot />
            </div>
          </Moveable>
        </div>
      </div>
    );
  },
);

export { Field as CarouselField } from './field';
