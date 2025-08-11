import { $, component$ } from '@builder.io/qwik';
import { isSameMonth } from 'date-fns';
import { CalendarGrid } from '../commons/calendar-grid';
import { CalendarGridButton } from '../commons/calendar-grid-button';
import { getFormattedDate, isDateInRange, Views } from '../date-picker-helpers';
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

export const CalendarViewMonth = component$(
  ({ minDate, maxDate, selectedDate, viewDate, language, view }: Props) => {
    const today = new Date();

    return (
      <CalendarGrid
        rows={3}
        cols={4}
        renderer={(row, col) => {
          const newDate = new Date(viewDate.value.getTime());
          newDate.setMonth(row * 4 + col);
          const month = getFormattedDate(language, newDate, { month: 'short' });

          const isSelected = selectedDate.value ? isSameMonth(selectedDate.value, newDate) : false;
          const isDisabled = !isDateInRange(newDate, minDate, maxDate);

          return (
            <CalendarGridButton
              name="month"
              highlight={isSameMonth(today, newDate)}
              key={`${row}-${col}`}
              isDisabled={isDisabled}
              isSelected={isSelected}
              onSelected$={$(() => {
                viewDate.value = newDate;
                view.value = Views.Days;
              })}
              label={month}
            />
          );
        }}
      />
    );
  },
);
