/** @type{import('tailwindcss').Config} */
const config = {
  content: ['**/index.html', './src/**/*.{ts,tsx}'],
  safelist: [
    {
      pattern: /bg-red/,
      variants: ['500']
    }
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'sans-serif']
      },
      colors: {
        bunker: {
          50: '#f7f7f8',
          100: '#eeeef0',
          200: '#d9d9de',
          300: '#b8b9c1',
          400: '#91939f',
          500: '#737584',
          600: '#5d5e6c',
          700: '#4c4d58',
          800: '#41414b',
          900: '#393941',
          950: '#18181b'
        },
        red: {
          50: '#fff0f1',
          100: '#ffdee0',
          200: '#ffc2c6',
          300: '#ff979e',
          400: '#ff5c66',
          500: '#ff2937',
          600: '#f90c1b',
          700: '#d20310',
          800: '#ad0712',
          900: '#8f0d15',
          950: '#4e0106'
        }
      }
    }
  },
  plugins: [require('@tailwindcss/forms'), require('@tailwindcss/typography')]
}

export default config
