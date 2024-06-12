import { twMerge } from 'tailwind-merge';
import { match } from 'ts-pattern';

import styles from './Tooltip.module.css';
import type { JSXChildren } from '@builder.io/qwik';
import type { Classes } from '~/utils/types';

export type TooltipPosition = 'right' | 'top' | 'left' | 'bottom' | 'bottom-left';

export type TooltipProps = {
  info: JSXChildren;
  children: JSXChildren;
  position?: TooltipPosition;
  hidden?: boolean;
  classes?: Classes<'root' | 'tooltip'>;
};

// TODO: remove tailwind styles
export const Tooltip = (props: TooltipProps) => (
  <div class={twMerge('relative w-fit h-fit', styles.TooltipContainer, props.classes?.root)}>
    {!props.hidden && (
      <div
        class={twMerge(
          'z-[1000] absolute px-2 py-0.5 transition-opacity delay-200 duration-500 opacity-0 rounded bg-space-1000 text-white pointer-events-none',

          match(props.position ?? 'right')
            .with('right', () => 'left-full -translate-x-[4px] top-1/2 -translate-y-1/2')
            .with('left', () => 'right-full translate-x-[4px] top-1/2 -translate-y-1/2')
            .with('top', () => 'bottom-full -translate-y-[4px] left-1/2 -translate-x-1/2')
            .with('bottom', () => 'top-full translate-y-[4px] left-1/2 -translate-x-1/2')
            .with('bottom-left', () => 'top-full right-full translate-y-[4px]')
            .exhaustive(),
          styles.Tooltip,
        )}
      >
        <div class={twMerge('text-sm tracking-wide py-0.5 px-1 w-max', props.classes?.tooltip)}>
          {props.info}
        </div>
      </div>
    )}
    {props.children}
  </div>
);
