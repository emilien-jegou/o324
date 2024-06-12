import { $, component$ } from '@builder.io/qwik';

import './global.css';
import { PortalLocation, PortalProvider } from './provider/portal-context';
import { TaskContextProvider } from './provider/task-context';
import { Calendar } from './routes/calendar';
import { Dashboard } from './routes/dashboard';
import { Debug } from './routes/debug';
import { Settings } from './routes/settings';
import { Stats } from './routes/stats';
import { SidebarLayout } from './ui/layout/sidebar-layout';
import {
  StateRouterPortal,
  StateRouterProvider,
  defineRouterContext,
} from './ui/logics/state-router';

export const routes = {
  dashboard: { load: $(() => <Dashboard class="w-full" />) },
  calendar: { load: $(() => <Calendar />) },
  stats: { load: $(() => <Stats />) },
  settings: { load: $(() => <Settings />) },
  debug: { load: $(() => <Debug />) },
};

export const routerContext = defineRouterContext<keyof typeof routes>();

export default component$(() => {
  return (
    <TaskContextProvider>
      <StateRouterProvider context={routerContext} defaultRoute="settings" routes={routes}>
        <head>
          <meta charSet="utf-8" />
          <link rel="manifest" href="/manifest.json" />
        </head>
        <body lang="en" class="bg-space-900 text-white">
          <PortalProvider>
            <div id="app-contents" class="flex w-screen">
              <SidebarLayout />
              <div style={{ width: 'calc(100vw - 144px)' }} class=" h-screen overflow-auto">
                <StateRouterPortal context={routerContext} />
              </div>
            </div>
            <PortalLocation zIndex={60} name="toast" />
            <PortalLocation zIndex={60} name="modal" />
            <PortalLocation zIndex={60} name="app-blur" />
            <PortalLocation zIndex={60} name="dropdown" />
          </PortalProvider>
        </body>
      </StateRouterProvider>
    </TaskContextProvider>
  );
});
