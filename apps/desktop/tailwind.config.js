import sharedConfig from "@tool-suite/ui/tailwind.config";

/** @type {import('tailwindcss').Config} */
export default {
	presets: [sharedConfig],
	content: [
		"./index.html",
		"./src/**/*.{js,ts,jsx,tsx}",
		"../../packages/ui/src/**/*.{ts,tsx}",
	],
};

