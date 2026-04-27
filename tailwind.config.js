/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: '#0a0a0a',
        foreground: '#f5f5f5',
        primary: '#171717',
        'primary-hover': '#262626',
        secondary: '#737373',
      }
    },
  },
  plugins: [],
}
