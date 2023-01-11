export interface DeckInfo {
	id: string;
	type: 'deck';
	name: string;
	deadline: Date | null;
}

export interface DeckFolderInfo {
	id: string;
	type: 'folder';
	name: string;
	contents: (DeckInfo | DeckFolderInfo)[] | null;
	open: boolean;
}
