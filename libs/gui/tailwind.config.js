/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{js,ts,jsx,tsx,mdx}'],
  theme: {
    extend: {
      colors: {
        'bg-subtle': '#f2f2f2',
        'bg-default': '#fff',

        'text-default': '#000',
        'text-subtle': '#6f6f6f',
        'text-subtler': '#cacaca',
        'text-disabled': '#c0c0c0',

        'border-default': '#181818',
        'border-subtle': '#cacaca',
        'border-subtler': '#ebebeb',

        'focused': 'rgba(0, 0, 0, 0.15)',
        'contrast': '#000',

        'space': {
          100: '#EEEFFC',
          200: '#E0E1EC',
          300: '#D2D3E0',
          400: '#9694A3',
          500: '#4C4F6B',
          600: '#444556',
          700: '#2A2B3A',
          800: '#1D1E2B',
          900: '#151621',
          1000: '#080808'
        },

        'violet': {
          DEFAULT: '#6C79FF',
          100: '#7477F0',
          200: '#6E79D6',
          300: '#5E6AD2',
          400: '#5C67C7',
          500: '#575BC7',
          600: '#37466C',
          700: '#2A2B51',
          800: '#222342'
        },

        'accent': {
          100: '#00B2BF',
          200: '#EB5757',
          300: '#FA6563',
          400: '#978200',
          500: '#F2C94C',
          600: '#F2994A',
          700: '#BB87FC',
          800: '#4EA7FC',
          900: '#95A2B3',
          1000: '#4CB782'
        }
      },
      borderWidth: {
        0.5: '0.5px',
      },
      outlineWidth: {
        0: '0px',
        2.5: '3px',
      },
    },
  },
  plugins: [require('tailwindcss-animate')],
};
