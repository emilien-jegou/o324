import type { JSXOutput, QRL } from '@builder.io/qwik';

type CalendarGridProps = {
  headerCategories?: { ['aria-label']: string; display: string }[];
  onKeyDown$?: QRL<(event: KeyboardEvent) => void>;
  rows: number;
  cols: number;
  renderer: (row: number, col: number) => JSXOutput;
};

export const CalendarGrid = ({ onKeyDown$, ...props }: CalendarGridProps) => (
  <table
    tabIndex={1}
    onKeyDown$={(e) => onKeyDown$?.(e)}
    class="w-full border-collapse"
    role="grid"
    aria-labelledby="calendar"
  >
    {props.headerCategories && (
      <thead class="text-space-400">
        <tr class="flex">
          {props.headerCategories.map((item, index) => (
            <th
              key={index}
              scope="col"
              class="text-muted-foreground rounded-md w-8 font-normal text-[0.8rem]"
              aria-label={item['aria-label']}
            >
              {item.display}
            </th>
          ))}
        </tr>
      </thead>
    )}

    <tbody class="" role="rowgroup">
      {Array.from({ length: props.rows }, (_, row) => {
        return (
          <tr key={row} class="flex w-full mt-2">
            {Array.from({ length: props.cols }, (_, col) => {
              return props.renderer(row, col);
            })}
          </tr>
        );
      })}
    </tbody>
  </table>
);
