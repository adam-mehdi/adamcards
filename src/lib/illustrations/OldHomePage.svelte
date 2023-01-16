<script lang="ts">
	import Greet from '../lib/Greet.svelte';
	import DeckList from '../lib/DeckList.svelte';
	import TestPostButton from '$lib/TestPostButton.svelte';
	import CreateDeckFromCsvButton from '$lib/CreateDeckFromCSVButton.svelte';
	import EditableCard from '$lib/EditableCard.svelte';
	import KaTeXRenderer from '$lib/KaTeXRenderer.svelte';
	import Folder from '$lib/Folder.svelte';
	// modes: normal and editing
	// - normal: go to review when clicking on deck, expand/collapse on deck
	// - editing: edit when clicking on deck, create new deck when clicking folder
	type FileSystemObject = {
		type: 'folder' | 'file';
		name: string;
		files?: FileSystemObject[];
	};
	// load contents of ~/data/decks (call to backend)
	let root: FileSystemObject[] = [
		{
			type: 'folder',
			name: 'Important work stuff',
			files: [{ type: 'file', name: 'quarterly-results.gif' }]
		},
		{
			type: 'folder',
			name: 'Animal GIFs',
			files: [
				{
					type: 'folder',
					name: 'Dogs',
					files: [
						{ type: 'file', name: 'treadmill.gif' },
						{ type: 'file', name: 'rope-jumping.gif' }
					]
				},
				{ type: 'file', name: 'cat-roomba.gif' }
			]
		},
		{ type: 'file', name: 'TODO.md' }
	];
</script>

<h1>MIO</h1>

<h2>Decks</h2>
<div class="folders">
	<Folder name="My Decks" files={root} expanded />
</div>

<h2>My Decks</h2>
<DeckList />
<hr />
<a href="/create-deck"><button>Create Deck</button></a>
<br />
<hr />
<h3>Editable Card</h3>
<a href="/demo">Svelte Examples</a>

<style>
	.folders {
		width: 100vw;
		max-width: 600px;
	}
</style>