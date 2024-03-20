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
    },
  },
  plugins: [],
};
