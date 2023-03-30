/** @type {import('tailwindcss').Config} */
module.exports = {
	content: [
    './src/**/*.{html,js,svelte,ts}', 
  ],
	darkMode: 'class',
	theme: {
		colors: {
			'columbia': "#B9D9ED",   // columbia blue
			'pink': "#F9A7B0",
			'inverted-columbia': "#462612",
			'columbia-dark': "#191970",   // midnight blue
			'offwhite': "#faf9f6", 	 // platinum from whiskey listings drop shadows
			'offblack': "#202020",   // same as obsidian theme background
			'whitetext': "#dcddde",  // white text from obsidian dark theme
			'blacktext': "#2e3338",  // black text from obsidian light theme
			"black":    "#0f0d0d",	 // black with a red tinge
			"white":    "#fff",       // straight white
			"platinum": "#e1dfdd",
			"slate-700": "#334155",
			"rose-300": "#fda4af",
			"green-400": "#4ade80",
			"cyan-700": "#0e7490"
		},
		extend: {}
	},
	plugins: [
		require('@tailwindcss/typography'),
	]
};