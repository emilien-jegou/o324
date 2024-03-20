type TaskSlotProps = {
  start: number;
  end: number;
  taskName: string;
  project: string;
  hexColor: string;
};

export const TaskSlot = (props: TaskSlotProps) => {
  const colors = getTaskColorsFromHex(props.hexColor);
  return (
    <div
      class="rounded-sm py-2 px-4 w-full text-sm"
      style={{ backgroundColor: colors.light, color: colors.dark }}
    >
      <p>
        <b class="font-semibold">{props.project}</b> {props.taskName}
      </p>
      <p class="text-xs mt-1">9:30 - 10:30</p>
    </div>
  );
};

function hexToHSL(hex: string) {
  // Convert hex to RGB first
  let r: number = 0,
    g: number = 0,
    b: number = 0;
  if (hex.length == 4) {
    r = parseInt(hex[1] + hex[1], 16);
    g = parseInt(hex[2] + hex[2], 16);
    b = parseInt(hex[3] + hex[3], 16);
  } else if (hex.length == 7) {
    r = parseInt(hex.substring(1, 3), 16);
    g = parseInt(hex.substring(3, 5), 16);
    b = parseInt(hex.substring(5, 7), 16);
  }

  // Then to HSL
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b),
    min = Math.min(r, g, b);

  let h: number = 0;
  let s: number = 0;
  const l: number = (max + min) / 2;

  if (max == min) {
    h = s = 0; // achromatic
  } else {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      case b:
        h = (r - g) / d + 4;
        break;
    }
    h /= 6;
  }

  return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) };
}

// Given a color, get it's light (bg) and dark (fg) version
const getTaskColorsFromHex = (hexColor: string) => {
  const res = hexToHSL(hexColor);

  return {
    light: `hsl(${res.h}, 79%, 83%)`,
    dark: `hsl(${res.h}, 55%, 19%)`,
  };
};
