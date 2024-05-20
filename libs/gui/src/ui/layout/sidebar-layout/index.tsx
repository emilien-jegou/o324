import { $, useContext, component$ } from '@builder.io/qwik';
import { routerContext } from '~/root';
import { Sidebar } from '~/ui/common/sidebar';
import { SidebarItem } from '~/ui/common/sidebar-item';

export const SidebarLayout = component$(() => {
  const context = useContext(routerContext);

  return (
    <Sidebar>
      <SidebarItem
        onClick$={$(() => {
          context.selected.value = 'dashboard';
        })}
        label="Dashboard"
      />
      <SidebarItem
        onClick$={$(() => {
          context.selected.value = 'calendar';
        })}
        label="Calendar"
      />
      <SidebarItem
        onClick$={$(() => {
          context.selected.value = 'stats';
        })}
        label="Statistics"
      />
      <SidebarItem
        onClick$={$(() => {
          context.selected.value = 'settings';
        })}
        label="Settings"
      />
      <SidebarItem
        onClick$={$(() => {
          context.selected.value = 'debug';
        })}
        label="Debug"
      />
    </Sidebar>
  );
});
