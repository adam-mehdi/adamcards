
import { writable } from "svelte/store";

interface EntryPair {
    parent_id: number;
    child_id: number;
}

interface Quota {
    new_left: number;
    review_left: number;
    num_progressed: number;
}

export interface EntryData {
    entry_id: number;
    entry_name: string;
    is_expanded?: boolean; // ? indicates field is Option<bool> in backend
    entry_type: string;
    entry_quota?: Quota;
}

export interface FolderSystem {
    pairs: EntryPair[];
    data: EntryData[];
}

const fs: FolderSystem = { pairs: [], data: [] };

export const folderSystemStore = writable(
    fs
);

export default folderSystemStore;