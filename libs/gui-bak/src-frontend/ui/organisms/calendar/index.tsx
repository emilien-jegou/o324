import { $, component$, useComputed$, useSignal } from '@builder.io/qwik';
import { IconButton } from '@/ui/common/icon-button';
import { ChevronLeftIcon } from '@/ui/icons/chevron-left';
import { ChevronRightIcon } from '@/ui/icons/chevron-right';
import { Carousel, CarouselField } from '@/ui/logics/carousel';
import { currentDate } from '@/utils/date';
import { CalendarDayButton } from './calendar-day-button';
import { CalendarView } from './calendar-view';

const MONTHS = [
  'January',
  'February',
  'March',
  'April',
  'May',
  'June',
  'July',
  'August',
  'September',
  'October',
  'November',
  'December',
];

export const Calendar = component$(() => {
  const start = useSignal(currentDate());
  const selected = useSignal(currentDate());
  const carouselPosition = useSignal<number | null>(null);

  const dates = useComputed$(() => {
    const currentDay = currentDate();

    // This is to make sure the slider always start on a monday
    const currentDayOffset = currentDay.getDay()
      ? currentDay.getDay() - 1
      : currentDay.getDay() + 6;

    const LEN = 60;
    return Array.from({ length: LEN }, (_, idx) => (LEN - 1 - idx) * 7).map((mul) =>
      Array.from({ length: 7 }, (_, idx: number) => {
        const date = new Date(start.value);
        date.setDate(date.getDate() + idx - currentDayOffset - mul);

        const dayCompare = (left: Date, right: Date) =>
          (left.getFullYear() - right.getFullYear()) * 100000 +
          (left.getMonth() - right.getMonth()) * 100 +
          (left.getDate() - right.getDate());

        return {
          date,
          selected: dayCompare(date, selected.value) == 0,
          disabled: dayCompare(date, currentDay) > 0,
        };
      }),
    );
  });

  const currentPosition = useComputed$(() => {
    return carouselPosition.value ?? dates.value.length - 1;
  });

  const currentMonth = useComputed$(() => {
    return MONTHS[dates.value[currentPosition.value][0].date.getMonth()];
  });

  return (
    <div class="flex flex-col h-full">
      <div class="py-5">
        <div class="flex items-center justify-between px-4 mb-2">
          <p class="font-light text-lg font-medium">{currentMonth.value}</p>
          <div>
            <IconButton
              onClick$={$((): void => {
                const newPosition = currentPosition.value - 1;

                if (newPosition >= 0) {
                  carouselPosition.value = newPosition;
                }
              })}
              icon={<ChevronLeftIcon />}
            />
            <IconButton
              onClick$={$((): void => {
                const newPosition = currentPosition.value + 1;

                if (newPosition < dates.value.length) {
                  carouselPosition.value = newPosition;
                }
              })}
              icon={<ChevronRightIcon />}
            />
          </div>
        </div>
        <div class="h-[3.4rem] overflow-x-hidden overflow-y-clip">
          <Carousel
            position={currentPosition.value}
            onPositionChange$={(newPosition: number) => {
              carouselPosition.value = newPosition;
            }}
            fieldQuantity={dates.value.length}
          >
            {dates.value.map((date, idx) => (
              <CarouselField position={idx} key={idx}>
                <div class="px-4 flex w-full justify-between">
                  {date.map((props: any) => (
                    <CalendarDayButton
                      {...props}
                      key={props.date}
                      onClick$={$((): void => {
                        selected.value = props.date;
                      })}
                    />
                  ))}
                </div>
              </CarouselField>
            ))}
          </Carousel>
        </div>
      </div>

      <hr />

      <CalendarView
        selected={selected.value}
        onSeePreviousDay$={() => {
          console.info('there');
          const current = new Date(selected.value);
          current.setDate(current.getDate() - 1);
          selected.value = current;
        }}
        onSeeNextDay$={() => {
          const current = new Date(selected.value);
          current.setDate(current.getDate() + 1);
          selected.value = current;
        }}
        class="h-full w-full"
      />
    </div>
  );
});
