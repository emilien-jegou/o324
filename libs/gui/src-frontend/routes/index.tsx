import { $, component$, useSignal } from '@builder.io/qwik';
import { Pane } from '@/ui/common/pane';
import { Tabs } from '@/ui/common/tabs';
import { Calendar } from '@/ui/organisms/calendar';
import { SearchMenu } from '@/ui/organisms/search-menu';
import type { DocumentHead } from '@builder.io/qwik-city';

const tabs = ['Timer', 'Calendar'] as const;

type TabName = (typeof tabs)[number];

// TODO: there is some weird behavior with tab navigation
export default component$(() => {
  const tab = useSignal<TabName>(tabs[1]);

  return (
    <div class="flex flex-col o3-h-screen w-screen">
      <div class="relative h-full w-full overflow-x-hidden">
        <Pane index={tabs.indexOf('Timer')} currentIndex={tabs.indexOf(tab.value)}>
          <div class="p-4">
            <p class="text-xs text-text-subtle">Start recording your progress</p>
            <SearchMenu class="mt-3" />
          </div>
        </Pane>
        <Pane index={tabs.indexOf('Calendar')} currentIndex={tabs.indexOf(tab.value)}>
          <Calendar />
        </Pane>
      </div>

      <div class="mt-auto pointer-events-none pb-2 px-2 absolute bottom-0 w-screen">
        <Tabs
          class="pointer-events-auto"
          selected={tab.value}
          options={tabs}
          onSelect$={$((option: TabName): void => {
            tab.value = option;
          })}
        />
      </div>
    </div>
  );
});

export const head: DocumentHead = {
  title: 'Welcome to Qwik',
  meta: [
    {
      name: 'description',
      content: 'Qwik site description',
    },
  ],
};
