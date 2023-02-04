/** @type {import('tailwindcss').Config} */
module.exports = {
	content: [
    './src/**/*.{html,js,svelte,ts}', 
  ],
	darkMode: 'class',
	theme: {
		colors: {
			'columbia': "#B9D9ED",   // columbia blue
			'inverted-columbia': "#462612",
			'columbia-dark': "#191970",   // midnight blue
			'offwhite': "#faf9f6", 	 // platinum from whiskey listings drop shadows
			'offblack': "#202020",   // same as obsidian theme background
			'whitetext': "#dcddde",  // white text from obsidian dark theme
			'blacktext': "#2e3338",  // black text from obsidian light theme
			"black":    "#0f0d0d",	 // black with a red tinge
			"white":    "#eee",       // straight white
			"platinum": "#e1dfdd"
		},
		extend: {}
	},
	plugins: []
};
