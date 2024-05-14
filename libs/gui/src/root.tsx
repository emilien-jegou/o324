import { component$ } from "@builder.io/qwik";

import "./global.css";
import { App } from "./app";

type SidebarItemProps = {
  label: string;
}

export const SidebarItem = (props: SidebarItemProps) => {
  return <button class="bg-gray-600 p-2 cursor-pointer hover:bg-gray-500 border border-gray-500">
    {props.label}
  </button>
}

export const Sidebar = () => {
  return <div class="flex gap-4 flex-col bg-gray-900 text-white p-4 h-screen">
    <SidebarItem label="Dashboard" />
    <SidebarItem label="Calendar" />
    <SidebarItem label="Settings" />
  </div>
}

export default component$(() => {
  return (
    <html>
      <head>
        <meta charSet="utf-8" />
        <link rel="manifest" href="/manifest.json" />
      </head>
      <body lang="en">
        <div class="flex w-screen">
          <Sidebar />
          <App class="w-full" />
        </div>
      </body>
    </html>
  );
});
