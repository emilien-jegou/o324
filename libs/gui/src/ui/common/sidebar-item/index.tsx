import type { PropFunction } from '@builder.io/qwik';

type SidebarItemProps = {
  label: string;
  onClick$?: PropFunction<() => void>;
};

export const SidebarItem = (props: SidebarItemProps) => {
  return (
    <button
      onClick$={props.onClick$}
      class="bg-space-800 px-4 py-2 cursor-pointer hover:bg-space-600 border border-space-600"
    >
      {props.label}
    </button>
  );
};

