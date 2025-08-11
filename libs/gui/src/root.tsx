import { component$ } from '@builder.io/qwik';

import './global.css';
import { QwikCityProvider, RouterOutlet, ServiceWorkerRegister } from '@builder.io/qwik-city';
import { PortalLocation, PortalProvider } from './provider/portal-context';
import { TaskContextProvider } from './provider/task-context';
import { SidebarLayout } from './ui/layout/sidebar-layout';

export default component$(() => {
  return (
    <QwikCityProvider>
      <head>
        <meta charSet="utf-8" />
        <link rel="manifest" href="/manifest.json" />
      </head>
      <body lang="en" class="bg-space-900 text-white">
        <TaskContextProvider>
          <PortalProvider>
            <div id="app-contents" class="flex w-screen">
              <SidebarLayout />
              <div style={{ width: 'calc(100vw - 144px)' }} class=" h-screen overflow-auto">
                <RouterOutlet />
                <ServiceWorkerRegister />
              </div>
            </div>
            <PortalLocation zIndex={60} name="toast" />
            <PortalLocation zIndex={60} name="modal" />
            <PortalLocation zIndex={60} name="app-blur" />
            <PortalLocation zIndex={60} name="dropdown" />
          </PortalProvider>
        </TaskContextProvider>
      </body>
    </QwikCityProvider>
  );
});
