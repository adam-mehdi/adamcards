# AdamCards

Here is the woefully undocumented repository for AdamCards! It is a desktop app created with Tauri.

Installation

npm install --save-dev @sveltejs/adapter-static@next

If there is no build directory in the root, run mkdir build for tauri. That is sufficient for the setup. Now to run the app for development, run

npm run tauri dev

Building

To build the app for distribution, run

npm run build
npm run tauri build

Make sure code signing is done
