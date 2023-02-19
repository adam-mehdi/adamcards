
import { writable } from "svelte/store";



const deleting = false;

const deletingStore = writable(
    deleting
);

export default deletingStore;