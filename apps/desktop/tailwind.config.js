/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: '#0B0F19',
        surface: '#111827',
        'surface-hover': '#1F2937',
        border: '#374151',
        primary: {
          DEFAULT: '#10B981', // Emerald
          hover: '#059669',
          glow: 'rgba(16, 185, 129, 0.25)',
        },
        accent: {
          blue: '#3B82F6',
          purple: '#8B5CF6',
          amber: '#F59E0B',
          rose: '#EF4444',
        }
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      }
    },
  },
  plugins: [],
}
