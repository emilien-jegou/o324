import { $, component$ } from '@builder.io/qwik';
import { isSameYear } from 'date-fns';
import { CalendarGrid } from '../commons/calendar-grid';
import { CalendarGridButton } from '../commons/calendar-grid-button';
import { isDateInRange, startOfYearPeriod, Views } from '../date-picker-helpers';
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

export const CalendarViewYear = component$(
  ({ selectedDate, minDate, maxDate, viewDate, view }: Props) => {
    return (
      <CalendarGrid
        rows={3}
        cols={4}
        renderer={(row, col) => {
          const first = startOfYearPeriod(viewDate.value, 10);
          const year = first - 1 + row * 4 + col;
          const newDate = new Date(viewDate.value.getTime());
          newDate.setFullYear(year);

          const isSelected = selectedDate.value ? isSameYear(selectedDate.value, newDate) : false;
          const isDisabled = !isDateInRange(newDate, minDate, maxDate);

          return (
            <CalendarGridButton
              name="year"
              highlight={new Date().getFullYear() == year}
              key={`${row}-${col}`}
              isSelected={isSelected}
              isDisabled={isDisabled}
              onSelected$={$(() => {
                viewDate.value = newDate;
                view.value = Views.Months;
              })}
              label={String(year)}
            />
          );
        }}
      />
    );
  },
);
