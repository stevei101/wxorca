/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // IBM Design Language colors
        ibm: {
          blue: {
            10: "#edf5ff",
            20: "#d0e2ff",
            30: "#a6c8ff",
            40: "#78a9ff",
            50: "#4589ff",
            60: "#0f62fe",
            70: "#0043ce",
            80: "#002d9c",
            90: "#001d6c",
            100: "#001141",
          },
          gray: {
            10: "#f4f4f4",
            20: "#e0e0e0",
            30: "#c6c6c6",
            40: "#a8a8a8",
            50: "#8d8d8d",
            60: "#6f6f6f",
            70: "#525252",
            80: "#393939",
            90: "#262626",
            100: "#161616",
          },
          teal: {
            50: "#08bdba",
            60: "#009d9a",
          },
          purple: {
            50: "#a56eff",
            60: "#8a3ffc",
          },
          red: {
            50: "#fa4d56",
            60: "#da1e28",
          },
          green: {
            50: "#24a148",
            60: "#198038",
          },
        },
      },
      fontFamily: {
        sans: [
          "IBM Plex Sans",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "sans-serif",
        ],
        mono: ["IBM Plex Mono", "ui-monospace", "monospace"],
      },
    },
  },
  plugins: [],
};
