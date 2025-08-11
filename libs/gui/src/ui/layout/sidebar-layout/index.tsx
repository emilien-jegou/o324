import { component$ } from '@builder.io/qwik';
import { Sidebar } from '~/ui/common/sidebar';
import { SidebarItem } from '~/ui/common/sidebar-item';

export const SidebarLayout = component$(() => {
  return (
    <Sidebar>
      <SidebarItem href="/" label="Dashboard" />
      <SidebarItem href="/calendar" label="Calendar" />
      <SidebarItem href="/stats" label="Statistics" />
      <SidebarItem href="/settings" label="Settings" />
      <SidebarItem href="/debug" label="Debug" />
    </Sidebar>
  );
});
