import { $, component$, useSignal, useTask$ } from '@builder.io/qwik';
import { isSameDay } from 'date-fns';
import { CalendarGrid } from '../commons/calendar-grid';
import { CalendarGridButton } from '../commons/calendar-grid-button';
import {
  addDays,
  getFirstDayOfTheMonth,
  getFormattedDate,
  getWeekDays,
  isDateInRange,
} from '../date-picker-helpers';
import type { Views, WeekStart } from '../date-picker-helpers';
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

export const CalendarViewDays = component$(
  ({
    weekStart,
    minDate,
    maxDate,
    viewDate,
    selectedDate,
    changeSelectedDate$,
    language,
  }: Props) => {
    const weekDays = getWeekDays(language, weekStart);
    const startDate = getFirstDayOfTheMonth(viewDate.value, weekStart);
    const today = new Date();
    const focusedDate = useSignal(selectedDate.value ?? today);
    const focusPressed = useSignal(false);

    useTask$(({ track }) => {
      track(() => selectedDate.value);
      if (selectedDate.value === undefined) {
        focusPressed.value = false;
        focusedDate.value = today;
      } else {
        focusedDate.value = selectedDate.value;
      }
    });

    const keyPress$ = $((pressedKey: string) => {
      if (pressedKey === 'ArrowDown') {
        const res = addDays(new Date(focusedDate.value), 7);
        focusedDate.value = res;
        focusPressed.value = true;
        viewDate.value = focusedDate.value;
      } else if (pressedKey === 'ArrowUp') {
        const res = addDays(new Date(focusedDate.value), -7);
        focusedDate.value = res;
        focusPressed.value = true;
        viewDate.value = focusedDate.value;
      } else if (pressedKey === 'ArrowLeft') {
        const res = addDays(new Date(focusedDate.value), -1);
        focusedDate.value = res;
        focusPressed.value = true;
        viewDate.value = focusedDate.value;
      } else if (pressedKey === 'ArrowRight') {
        const res = addDays(new Date(focusedDate.value), 1);
        focusedDate.value = res;
        focusPressed.value = true;
        viewDate.value = focusedDate.value;
      } else if (pressedKey === 'Enter') {
        changeSelectedDate$(focusedDate.value);
      }
    });

    return (
      <CalendarGrid
        headerCategories={weekDays.map((day) => ({ 'aria-label': day.long, display: day.short }))}
        rows={6}
        cols={7}
        onKeyDown$={$((event: KeyboardEvent) => {
          keyPress$(event.key);
        })}
        renderer={(row, col) => {
          const currentDate = addDays(startDate, row * 7 + col);
          const day = getFormattedDate(language, currentDate, { day: 'numeric' });

          const isSelected = selectedDate.value
            ? isSameDay(selectedDate.value, currentDate)
            : false;

          const isDisabled = !isDateInRange(currentDate, minDate, maxDate);
          return (
            <CalendarGridButton
              name="day"
              highlight={isSameDay(today, currentDate)}
              dim={currentDate.getMonth() !== viewDate.value.getMonth()}
              key={`${row}-${col}`}
              isDisabled={isDisabled}
              isSelected={isSelected}
              isFocused={focusPressed.value === true && isSameDay(focusedDate.value, currentDate)}
              onSelected$={$(() => {
                const currentSelected = selectedDate.value;
                if (currentSelected !== undefined) {
                  currentDate.setHours(currentSelected.getHours());
                  currentDate.setMinutes(currentSelected.getMinutes());
                }
                changeSelectedDate$(currentDate);
              })}
              label={day}
            />
          );
        }}
      />
    );
  },
);
