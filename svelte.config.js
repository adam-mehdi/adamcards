import preprocess from 'svelte-preprocess';
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/kit/vite';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://kit.svelte.dev/docs/integrations#preprocessors
	// for more information about preprocessors
	preprocess: [
		vitePreprocess(),
		preprocess({
			postcss: true
		})
	],

	kit: {
		adapter: adapter(),

     	// prerender: { default: true },
		// adapter: adapter({
		// 	fallback: 'failure.html'
		// }),
		// prerender: { 
		// 	entries: [
		// 		"*"
		// 	"/src/routes/+page.svelte",
		// 	"/src/routes/[param]/edit/+page.svelte",
		// 	"/src/routes/[param]/review/+page.svelte",
		// 	]
		// }
	}
};

export default config;
