import { SvgIcon } from '../common/svg-icon';
import type { IconProps } from '../common/svg-icon';

export const StopIcon = (props: IconProps) => (
  <SvgIcon viewBox="0 0 16 16" {...props}>
    <path d="M0 16V0H16V16H0Z" fill="currentColor" />
  </SvgIcon>
);
