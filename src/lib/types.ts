
	
export type FileSystemObject = {
    entity_type: 'folder' | 'deadline' | 'deck';
    name: string;
    files: FileSystemObject[] | null;
    expanded: boolean | null;
    deadline_date: string | null;
    deadline_time: string | null;
};