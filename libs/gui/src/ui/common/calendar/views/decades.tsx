import { $, component$ } from '@builder.io/qwik';
import { CalendarGrid } from '../commons/calendar-grid';
import { CalendarGridButton } from '../commons/calendar-grid-button';
import { addYears, startOfYearPeriod, Views } from '../date-picker-helpers';
import type { WeekStart } from '../date-picker-helpers';
import type { QRL, Signal } from '@builder.io/qwik';

type Props = {
  language: string;
  weekStart: WeekStart;
  minDate?: Date;
  maxDate?: Date;
  view: Signal<Views>;
  selectedDate: Signal<Date | undefined>;
  viewDate: Signal<Date>;
  changeSelectedDate$: QRL<(date: Date) => void>;
};

export const CalendarViewDecade = component$(
  ({ minDate, maxDate, view, viewDate, selectedDate }: Props) => {
    const today = new Date();

    return (
      <CalendarGrid
        rows={3}
        cols={4}
        renderer={(row, col) => {
          const first = startOfYearPeriod(viewDate.value, 100);
          const year = first - 10 + (row * 4 + col) * 10;
          const firstDate = new Date(year, 0, 1);
          const lastDate = addYears(firstDate, 9);

          const isSelected =
            !!selectedDate.value && firstDate < selectedDate.value && lastDate > selectedDate.value;
          const isEnabled =
            (minDate ? minDate > firstDate : true) && (maxDate ? maxDate < lastDate : true);

          return (
            <CalendarGridButton
              name="decade"
              highlight={firstDate < today && lastDate > today}
              key={`${row}-${col}`}
              isSelected={isSelected}
              isDisabled={!isEnabled}
              onSelected$={$(() => {
                const d = new Date(firstDate);
                d.setFullYear(d.getFullYear() + 1);
                viewDate.value = d;
                view.value = Views.Years;
              })}
              label={String(year)}
            />
          );
        }}
      />
    );
  },
);
