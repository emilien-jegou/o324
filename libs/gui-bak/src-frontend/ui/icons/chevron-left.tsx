import { SvgIcon } from '../common/svg-icon';
import type { IconProps } from '../common/svg-icon';

export const ChevronLeftIcon = (props: IconProps) => (
  <SvgIcon viewBox="0 0 16 16" {...props}>
    <path
      fill="currentColor"
      d="M10.354 3.146a.5.5 0 0 1 0 .708L6.207 8l4.147 4.146a.5.5 0 0 1-.708.708l-4.5-4.5a.5.5 0 0 1 0-.708l4.5-4.5a.5.5 0 0 1 .708 0"
    />
  </SvgIcon>
);
