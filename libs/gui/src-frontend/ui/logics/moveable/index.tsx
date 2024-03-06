import { $, type PropFunction, component$, useOnWindow, Slot, useSignal } from '@builder.io/qwik';

type MoveableProps = {
  axis: 'x' | 'y' | 'xy';
  onMoveEnd$?: PropFunction<(data: MouseMoveData & { end: Date }) => void>;
};

type Position = {
  x: number;
  y: number;
};

type MoveType = 'touch' | 'mouse';

type MouseMoveData = {
  moveType: MoveType;
  offsetCoef: Position;
  translate: Position;
  initialPos: Position;
  initialTranslate: Position;
  start: Date;
};

// TODO: we need resize observer
export const Moveable = component$((props: MoveableProps) => {
  const moveData = useSignal<MouseMoveData | null>(null);
  const containerRef = useSignal<HTMLDivElement>();

  useOnWindow(
    'mousemove',
    $((event) => {
      if (moveData.value?.moveType !== 'mouse' || !containerRef.value) return;

      // Get the position of the div element
      const rect = containerRef.value.getBoundingClientRect();

      // Calculate the touch position inside the div
      const x = event.clientX - moveData.value.initialPos.x;
      const y = event.clientY - moveData.value.initialPos.y;

      // See previous position using offset
      const prevX = rect.width * moveData.value.offsetCoef.x;
      const prevY = rect.height * moveData.value.offsetCoef.y;

      const translateX = props.axis.includes('x') ? x - prevX : 0;
      const translateY = props.axis.includes('y') ? y - prevY : 0;

      moveData.value.translate = { x: translateX, y: translateY };

      containerRef.value.style.transform = `translateX(${translateX}px) translateY(${translateY}px)`;
    }),
  );

  useOnWindow(
    'mouseup',
    $(() => {
      if (moveData.value?.moveType !== 'mouse') return;
      props.onMoveEnd$?.({ ...moveData.value, end: new Date() });
      moveData.value = null;
    }),
  );

  return (
    <div
      ref={containerRef}
      class="select-none"
      onMouseDown$={(event) => {
        if (moveData.value !== null || !containerRef.value) return;

        // Get the position of the div element
        const rect = containerRef.value!.getBoundingClientRect();

        // Calculate the mouse position inside the div (relative to the top-left corner of the div)
        const x = (event.clientX - rect.x) / rect.width;
        const y = (event.clientY - rect.y) / rect.height;

        // Take into account existing translate on the element to calculate initial position.
        const style = window.getComputedStyle(containerRef.value);
        const matrix = new DOMMatrix(style.transform);

        const translateX = matrix.m41;
        const translateY = matrix.m42;

        moveData.value = {
          moveType: 'mouse',
          offsetCoef: { x, y },
          translate: { x: translateX, y: translateY },
          initialTranslate: { x: translateX, y: translateY },
          initialPos: { x: rect.x - translateX, y: rect.y - translateY },
          start: new Date(),
        };
      }}
      onTouchStart$={(event) => {
        if (moveData.value !== null || !containerRef.value) return;
        // Get the first touch point
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        const touch = event.touches[0] || event.changedTouches[0];

        // Get the position of the div element
        const rect = containerRef.value!.getBoundingClientRect();

        // Calculate the touch position inside the div (relative to the top-left corner of the div)
        const x = (touch.pageX - rect.x) / rect.width;
        const y = (touch.pageY - rect.y) / rect.height;

        // Take into account existing translate on the element to calculate initial position.
        const style = window.getComputedStyle(containerRef.value);
        const matrix = new DOMMatrix(style.transform);

        const translateX = matrix.m41;
        const translateY = matrix.m42;

        moveData.value = {
          moveType: 'touch',
          offsetCoef: { x, y },
          translate: { x: translateX, y: translateY },
          initialTranslate: { x: translateX, y: translateY },
          initialPos: { x: rect.x - translateX, y: rect.y - translateY },
          start: new Date(),
        };
      }}
      onTouchMove$={(event) => {
        if (moveData.value?.moveType !== 'touch' || !containerRef.value) return;

        // Get the first touch point
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        const touch = event.touches[0] || event.changedTouches[0];

        // Get the position of the div element
        const rect = containerRef.value.getBoundingClientRect();

        // Calculate the touch position inside the div
        const x = touch.pageX - moveData.value.initialPos.x;
        const y = touch.pageY - moveData.value.initialPos.y;

        // See previous position using offset
        const prevX = rect.width * moveData.value.offsetCoef.x;
        const prevY = rect.height * moveData.value.offsetCoef.y;

        const translateX = props.axis.includes('x') ? x - prevX : 0;
        const translateY = props.axis.includes('y') ? y - prevY : 0;

        moveData.value.translate = { x: translateX, y: translateY };

        containerRef.value.style.transform = `translateX(${translateX}px) translateY(${translateY}px)`;
      }}
      onTouchEnd$={() => {
        if (moveData.value?.moveType !== 'touch') return;
        props.onMoveEnd$?.({ ...moveData.value, end: new Date() });
        moveData.value = null;
      }}
    >
      <Slot />
    </div>
  );
});
