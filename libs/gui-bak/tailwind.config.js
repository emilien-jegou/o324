/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src-frontend/**/*.{js,ts,jsx,tsx,mdx}'],
  theme: {
    extend: {
      borderWidth: {
        0.5: '0.5px',
      },
      outlineWidth: {
        0: '0px',
        2.5: '3px',
      },
      colors: {
        // Background
        'bg-default': 'var(--color-bg-default)',
        'bg-subtle': 'var(--color-bg-subtle)',

        // Text
        'text-default': 'var(--color-text-default)',
        'text-subtle': 'var(--color-text-subtle)',
        'text-subtler': 'var(--color-text-subtler)',
        'text-disabled': 'var(--color-text-disabled)',

        // Border
        'border-default': 'var(--color-border-default)',
        'border-subtle': 'var(--color-border-subtle)',
        'border-subtler': 'var(--color-border-subtler)',

        // Other
        focused: 'var(--color-focused)',
        'contrast': 'var(--color-contrast)',
      },
    },
  },
  plugins: [],
};
