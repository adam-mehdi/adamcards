import { writable } from "svelte/store";


type FileSystemObject = {
    entity_type: 'folder' | 'deadline' | 'deck';
    name: string;
    files: FileSystemObject[] | null;
    expanded: boolean | null;
    deadline_date: string | null;
    deadline_time: string | null;
};

const fs: FileSystemObject[] = []

const fsStore = writable(
   fs 
);

export default fsStore;