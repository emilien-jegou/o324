const CUSTOM_CURRENT_DATE: Date | null = null;

// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
export const currentDate = (): Date => CUSTOM_CURRENT_DATE ?? new Date();
