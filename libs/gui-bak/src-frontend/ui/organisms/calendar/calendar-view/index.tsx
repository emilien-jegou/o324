import { useComputed$, type PropFunction, component$ } from '@builder.io/qwik';
import { twMerge } from 'tailwind-merge';
import { Button } from '@/ui/common/button';
import { ArrowLeftIcon } from '@/ui/icons/arrow-left';
import { ArrowRightIcon } from '@/ui/icons/arrow-right';
import { HourMarker } from './hour-marker';
import { TaskSlot } from './task-slot';

type CalendarViewProps = {
  class?: string;
  selected: Date;
  onSeePreviousDay$: PropFunction<() => void>;
  onSeeNextDay$: PropFunction<() => void>;
};

const isToday = (dateToCheck: Date) => {
  const today = new Date();
  return (
    dateToCheck.getDate() === today.getDate() &&
    dateToCheck.getMonth() === today.getMonth() &&
    dateToCheck.getFullYear() === today.getFullYear()
  );
};

export const CalendarView = component$((props: CalendarViewProps) => {
  const dateSelectedIsToday = useComputed$(() => isToday(props.selected));

  return (
    <div class={twMerge(props.class, 'overflow-y-auto pb-16')}>
      <div class="px-4 py-4 w-full">
        <Button
          onClick$={props.onSeePreviousDay$}
          class="w-full"
          label="Previous day"
          iconStart={<ArrowLeftIcon />}
        />
      </div>
      <div class="relative mt-4 grid grid-cols-[24] h-[1200px]">
        {Array.from({ length: 24 }, (_, idx) => (
          <HourMarker key={idx} time={String(idx) + ' AM'} />
        ))}
        <div class="pl-16 pr-5 w-full absolute grid grid-cols[288]">
          <TaskSlot hexColor="#FA00FF" start={40} end={80} taskName="My activity" project="Work" />
        </div>
      </div>
      <HourMarker time="0 AM" />
      {/* TODO: Automatically cycle to next/previous day without the need of button click */}

      <div class="px-4 pt-4 w-full">
        <Button
          onClick$={props.onSeeNextDay$}
          label="Next day"
          class="w-full"
          disabled={dateSelectedIsToday.value}
          iconEnd={<ArrowRightIcon />}
        />
      </div>
    </div>
  );
});
