export interface DeckInfo {
	id: number;
	type: 'deck';
	name: string;
	deadline: Date;
}

export interface DeckFolderInfo {
	id: number;
	type: 'folder';
	name: string;
	contents: (DeckInfo | DeckFolderInfo)[] | null;
	open: boolean;
}
