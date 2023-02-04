
import { writable } from "svelte/store";


type EntryQuota = {
    new_left: number,
    review_left: number,
    num_progressed: number,
    days_to_go: number,
    tot_days: number,
    deck_path: string
};

const quota: EntryQuota[] = [];

const quotaStore = writable(
   quota 
);

export default quotaStore;