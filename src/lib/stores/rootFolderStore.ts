
import { writable } from "svelte/store";

const root_folders: number[] = [];

export const rootFolderStore = writable(
    root_folders
);

export default rootFolderStore;