import type { PropFunction } from '@builder.io/qwik';
import { $, component$, useContext } from '@builder.io/qwik';

import './global.css';
import { Calendar } from './routes/calendar';
import { Dashboard } from './routes/dashboard';
import { Settings } from './routes/settings';
import {
  StateRouterPortal,
  StateRouterProvider,
  defineRouterContext,
} from './ui/logics/state-router';

type SidebarItemProps = {
  label: string;
  onClick$?: PropFunction<() => void>;
};

export const SidebarItem = (props: SidebarItemProps) => {
  return (
    <button
      onClick$={props.onClick$}
      class="bg-gray-600 px-4 py-2 cursor-pointer hover:bg-gray-500 border border-gray-500"
    >
      {props.label}
    </button>
  );
};

export const Sidebar = component$(() => {
  const context = useContext(routerContext);

  return (
    <div class="flex gap-4 flex-col bg-gray-900 text-white p-4 h-screen">
      <SidebarItem
        onClick$={$(() => {
          context.selected.value = 'dashboard';
        })}
        label="Dashboard"
      />
      <SidebarItem
        onClick$={$(() => {
          console.info('calendar');
          context.selected.value = 'calendar';
        })}
        label="Calendar"
      />
      <SidebarItem
        onClick$={$(() => {
          context.selected.value = 'settings';
        })}
        label="Settings"
      />
    </div>
  );
});

export const routes = {
  dashboard: { load: $(() => <Dashboard class="w-full" />) },
  calendar: { load: $(() => <Calendar />) },
  settings: { load: $(() => <Settings />) },
};

export const routerContext = defineRouterContext<keyof typeof routes>();

export default component$(() => {
  return (
    <html>
      <StateRouterProvider context={routerContext} defaultRoute="dashboard" routes={routes}>
        <head>
          <meta charSet="utf-8" />
          <link rel="manifest" href="/manifest.json" />
        </head>
        <body lang="en">
          <div class="flex w-screen">
            <Sidebar />
            <StateRouterPortal context={routerContext} />
          </div>
        </body>
      </StateRouterProvider>
    </html>
  );
});
