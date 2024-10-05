/** @type{import('tailwindcss').Config} */
const config = {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
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
          50: '#FFFFFF',
          100: '#FFFFFF',
          200: '#ECECEE',
          300: '#B9B9C1',
          400: '#858593',
          500: '#585863',
          600: '#4A4A54',
          700: '#3C3C44',
          800: '#2E2E33',
          900: '#222226',
          950: '#1A1A1E'
        },

        atlantis: {
          50: '#f6fbea',
          100: '#eaf6d1',
          200: '#d5eda9',
          300: '#b9df77',
          400: '#9dcf4c',
          500: '#8cc733',
          600: '#628f21',
          700: '#4b6e1d',
          800: '#3e571d',
          900: '#354b1c',
          950: '#1a290a'
        },

        pelorous: {
          50: '#edfdfe',
          100: '#d2f9fb',
          200: '#aaf1f7',
          300: '#70e5f0',
          400: '#2ecfe2',
          500: '#11a7bb',
          600: '#128fa8',
          700: '#167388',
          800: '#1b5d6f',
          900: '#1b4e5e',
          950: '#0c3240'
        }
      }
    }
  },
  plugins: [require('@tailwindcss/forms'), require('@tailwindcss/typography')]
}

export default config
