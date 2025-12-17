import type { Config } from "tailwindcss";
import sharedConfig from "@tool-suite/ui/tailwind.config";

const config: Config = {
  presets: [sharedConfig],
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
    "../../packages/ui/src/**/*.{ts,tsx}",
  ],
};
export default config;
