import { $, component$, useSignal, useTask$ } from '@builder.io/qwik';
import { LibIcon } from '~/ui/icons/lib-icon';
import { cn } from '~/utils/cn';
import {
  addMonths,
  addYears,
  getFirstDateInRange,
  getFormattedDate,
  startOfYearPeriod,
  Views,
  WeekStart,
} from './date-picker-helpers';
import { TimeModule } from './time-module';
import { CalendarViewDays } from './views/days';
import { CalendarViewDecade } from './views/decades';
import { CalendarViewMonth } from './views/months';
import { CalendarViewYear } from './views/years';
import type { JSXChildren, PropFunction, QRL, Signal } from '@builder.io/qwik';
import type { Prettify, XOR } from 'ts-essentials';

export type CalendarProps = {
  ref?: Signal<HTMLDivElement | undefined>;
  class?: string;
} & GenericCalendarProps;

export type GenericCalendarProps = Prettify<
  {
    showClearButton?: boolean;
    labelClearButton?: string;
    showTodayButton?: boolean;
    labelTodayButton?: string;
    language?: string;
    ['bind:value']: Signal<Date | undefined>;
    onSelected$?: QRL<(date: Date | undefined) => void>;
    minDate?: Date;
    maxDate?: Date;
    weekStart?: WeekStart;
    mode?: 'date' | 'date-and-time';
  } & XOR<
    { mode?: 'date'; options?: {} },
    {
      mode: 'date-and-time';
      options?: {
        // Default minutes increment is 5
        minutesIncrement: number;
      };
    }
  >
>;

export const Calendar = component$(
  ({
    ref,
    showClearButton,
    labelClearButton = 'Clear',
    showTodayButton,
    labelTodayButton = 'Today',
    minDate,
    maxDate,
    language = 'en',
    weekStart = WeekStart.Sunday,
    onSelected$,
    mode,
    ...props
  }: CalendarProps) => {
    const value = props['bind:value'];
    const viewDefaultDate = getFirstDateInRange(
      props['bind:value'].value ?? new Date(),
      minDate,
      maxDate,
    );
    const view = useSignal(Views.Days);
    const viewDate = useSignal(viewDefaultDate);

    // Triggers when user select the date
    const changeSelectedDate$ = $(async (date: Date | undefined) => {
      value.value = date;
      if (onSelected$) {
        await onSelected$(date);
      }
    });

    // Update the view on date change
    useTask$(({ track }) => {
      track(() => value.value);
      viewDate.value = value.value ?? viewDefaultDate;
    });

    // Render the DatePickerView* node
    const renderView = (type: Views) => {
      const renderProps = {
        language,
        minDate,
        maxDate,
        weekStart,
        view,
        viewDate,
        selectedDate: props['bind:value'],
        changeSelectedDate$: changeSelectedDate$,
      };
      switch (type) {
        case Views.Decades:
          return <CalendarViewDecade {...renderProps} />;
        case Views.Years:
          return <CalendarViewYear {...renderProps} />;
        case Views.Months:
          return <CalendarViewMonth {...renderProps} />;
        case Views.Days:
        default:
          return <CalendarViewDays {...renderProps} />;
      }
    };

    // Coordinate the next view based on current view (statemachine-like)
    const getNextView$ = $(() => {
      switch (view.value) {
        case Views.Days:
          return Views.Months;
        case Views.Months:
          return Views.Years;
        case Views.Years:
          return Views.Decades;
      }
      return view.value;
    });

    // Get the view title based on active View
    const getViewTitle = () => {
      switch (view.value) {
        case Views.Decades:
          return `${startOfYearPeriod(viewDate.value, 100)} - ${startOfYearPeriod(viewDate.value, 100) + 90}`;
        case Views.Years:
          return `${startOfYearPeriod(viewDate.value, 10)} - ${startOfYearPeriod(viewDate.value, 10) + 9}`;
        case Views.Months:
          return getFormattedDate(language, viewDate.value, { year: 'numeric' });
        case Views.Days:
        default:
          return getFormattedDate(language, viewDate.value, { month: 'long', year: 'numeric' });
      }
    };

    // Navigate to prev/next for given view's date by value
    const getViewDatePage$ = $((view: Views, date: Date, value: number) => {
      switch (view) {
        case Views.Days:
          return new Date(addMonths(date, value));
        case Views.Months:
          return new Date(addYears(date, value));
        case Views.Years:
          return new Date(addYears(date, value * 10));
        case Views.Decades:
          return new Date(addYears(date, value * 100));
        default:
          return new Date(addYears(date, value * 10));
      }
    });

    return (
      <div
        ref={ref}
        class={cn(
          'relative rounded-md flex px-4 items-stretch w-fit shadow-lg border border-space-700 text-white bg-space-1000',
          props.class,
        )}
      >
        <div class="block w-[232px] shrink-0 py-4 pt-3">
          <div>
            <div
              tabIndex={1}
              onKeyDown$={$(async (event) => {
                if (event.key === 'ArrowLeft') {
                  viewDate.value = await getViewDatePage$(view.value, viewDate.value, -1);
                } else if (event.key === 'ArrowRight') {
                  viewDate.value = await getViewDatePage$(view.value, viewDate.value, +1);
                }
              })}
              aria-label="month"
              class="mb-2 flex items-center justify-between"
            >
              <HeadNavButton
                icon={<LibIcon icon="chevron-left" />}
                onClick$={$(async () => {
                  viewDate.value = await getViewDatePage$(view.value, viewDate.value, -1);
                })}
              />

              <button
                type="button"
                tabIndex={-1}
                class="rounded-md px-3 py-1.5 text-sm font-medium hover:bg-space-900/60"
                onClick$={async () => (view.value = await getNextView$())}
              >
                {getViewTitle()}
              </button>

              <HeadNavButton
                icon={<LibIcon icon="chevron-right" />}
                onClick$={$(async () => {
                  viewDate.value = await getViewDatePage$(view.value, viewDate.value, +1);
                })}
              />
            </div>
          </div>
          <div class="p-1">{renderView(view.value)}</div>
          {(showClearButton || showTodayButton) && (
            <div class="mt-2 flex space-x-2">
              {showTodayButton && (
                <button
                  type="button"
                  class={cn(
                    'w-full rounded-lg px-5 py-2 text-center text-sm font-medium focus:ring-4',
                    'focus:ring-blue-300',
                  )}
                  onClick$={$(() => {
                    const today = new Date();
                    changeSelectedDate$(today);
                    viewDate.value = today;
                  })}
                >
                  {labelTodayButton}
                </button>
              )}
              {showClearButton && (
                <button
                  type="button"
                  class={cn(
                    'w-full rounded-lg px-5 py-2 text-center text-sm font-medium focus:ring-4',
                    'focus:ring-blue-300',
                    'border',
                  )}
                  onClick$={() => {
                    changeSelectedDate$(undefined);
                  }}
                >
                  {labelClearButton}
                </button>
              )}
            </div>
          )}
        </div>
        {mode === 'date-and-time' && (
          <TimeModule
            selected={value.value}
            onTimeChange$={$((hours: number, minutes: number) => {
              const current = new Date(value.value!);
              current.setHours(hours);
              current.setMinutes(minutes);
              changeSelectedDate$(current);
            })}
            currentView={view.value}
          />
        )}
      </div>
    );
  },
);

type HeadNavButtonProps = {
  icon: JSXChildren;
  onClick$: PropFunction<() => void>;
};

export const HeadNavButton = (props: HeadNavButtonProps) => (
  <button
    type="button"
    tabIndex={-1}
    class="rounded-md text-space-400 border border-space-800 flex items-center justify-center w-7 h-7 transition-colors hover:bg-space-700 hover:text-white"
    onClick$={props.onClick$}
  >
    {props.icon}
  </button>
);
