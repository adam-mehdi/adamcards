## AdamCards

Welcome to AdamCards, a spaced-reptition that cares about your life.

Contact me for the desktop app distribution.

## Installation

Cardway is a tauri app that uses Svelte as its frontend and SvelteKit as its router. Install SvelteKit for tauri with the following.

```
npm install --save-dev @sveltejs/adapter-static@next
```



If there is no `build` directory in the root, run `mkdir build` for tauri. That is sufficient for the setup. Now to run the app for development, run

```
npm run tauri dev
```



## Building

To build the app for distribution, run

```
npm run build
npm run tauri build
```

Make sure code signing is done
