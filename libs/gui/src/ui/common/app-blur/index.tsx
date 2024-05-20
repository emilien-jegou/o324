import { $ } from '@builder.io/qwik';
import { Portal } from '~/provider/portal-context';
import type { PropFunction } from '@builder.io/qwik';

type AppBlurProps = {
  onClick$?: PropFunction<() => void>;
};
// TODO: fix this:
export const AppBlur = (props: AppBlurProps) => (
  <Portal
    name="app-blur"
    render$={$(() => (
      <div
        onClick$={props.onClick$}
        class="z-50 w-screen h-screen absolute top-0 left-0 bg-black opacity-60 pointer-events-none select-none"
      />
    ))}
  />
);
