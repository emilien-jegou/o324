import type { JSXChildren } from '@builder.io/qwik';

type SidebarProps = {
  children: JSXChildren;
};

export const Sidebar = (props: SidebarProps) => (
  <div class="flex gap-4 flex-col bg-space-1000 text-white p-4 h-screen">{props.children}</div>
);
