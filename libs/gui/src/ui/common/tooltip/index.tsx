import { twMerge } from 'tailwind-merge';
import { match } from 'ts-pattern';

import styles from './Tooltip.module.css';
import type { JSXChildren } from '@builder.io/qwik';
import type { Classes } from '~/utils/types';

export type TooltipProps = {
  info: JSXChildren;
  children: JSXChildren;
  position?: 'right' | 'top';
  hidden?: boolean;
  classes?: Classes<'root' | 'tooltip'>;
};

// TODO: remove tailwind styles
export const Tooltip = (props: TooltipProps) => (
  <div class={twMerge('relative w-fit h-fit', styles.TooltipContainer, props.classes?.root)}>
    {!props.hidden && (
      <div
        class={twMerge(
          'absolute px-2 py-0.5 transition-opacity delay-200 duration-500 opacity-0 rounded bg-base-black text-base-white pointer-events-none',

          match(props.position ?? 'right')
            .with('right', () => 'left-full top-1/2 -translate-y-1/2')
            .with('top', () => 'bottom-full left-1/2 -translate-x-1/2')
            .exhaustive(),
          styles.Tooltip,
        )}
      >
        <div class={twMerge('text-xs tracking-wide font-medium w-max', props.classes?.tooltip)}>
          {props.info}
        </div>
      </div>
    )}
    {props.children}
  </div>
);
