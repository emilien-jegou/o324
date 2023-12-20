import { component$, useSignal } from '@builder.io/qwik';
import { ChevronLeftIcon } from '@/ui/icons/chevron-left';
import { ChevronRightIcon } from '@/ui/icons/chevron-right';
import { CalendarDayButton } from './calendar-day-button';

export const Calendar = component$(() => {
  const selected = useSignal(new Date());

  return (
    <div>
      <div class="flex justify-between">
        <ChevronLeftIcon />
        <p class="font-light text-sm">2023, October, Week 45</p>
        <ChevronRightIcon />
      </div>
      <hr class="my-4" />
      <div class="flex w-full justify-between">
        {Array.from({ length: 7 }, (_, idx: number) => {
          const date = new Date();
          const offset = idx - 4;
          date.setDate(date.getDate() + idx - 4);
          return (
            <CalendarDayButton
              key={idx}
              date={date}
              selected={offset === 0}
              disabled={offset > 0}
            />
          );
        })}
      </div>
    </div>
  );
});
