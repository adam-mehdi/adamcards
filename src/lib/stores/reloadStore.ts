
import { writable } from "svelte/store";



const reloadToggle = false;

const reloadStore = writable(
    reloadToggle
);

export default reloadStore;