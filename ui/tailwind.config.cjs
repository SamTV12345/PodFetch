/** @type {import('tailwindcss').Config} */

const defaultTheme = require('tailwindcss/defaultTheme')

module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    screens: {
      'xs': '480px',
      ...defaultTheme.screens,
    },
    extend: {
      colors: {
        mustard: {
          100: '#fae5c0',
          200: '#fad490',
          300: '#fac463',
          400: '#f3ae34',
          500: '#e69a13',
          600: '#c07c03',
          700: '#855602',
          800: '#4e3201',
          900: '#271901',
        },
      },
    },
  },
  plugins: [],
}
