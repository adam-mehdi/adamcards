<script lang="ts">
	import Greet from '../lib/Greet.svelte';
	import DeckList from '../lib/DeckList.svelte';
	import TestPostButton from '$lib/TestPostButton.svelte';
	import CreateDeckFromCsvButton from '$lib/CreateDeckFromCSVButton.svelte';
	import EditableCard from '$lib/EditableCard.svelte';
	import KaTeXRenderer from '$lib/KaTeXRenderer.svelte';
	import Folder from '$lib/Folder.svelte';


	// load contents of ~/data/decks recursivey
	type FileSystemObject = {
		type: 'folder' | 'file',
		name: string,
		files?: FileSystemObject[]
	}
	let root: FileSystemObject[] = [
		{
			type: 'folder',
			name: 'Important work stuff',
			files: [
				{ type: 'file', name: 'quarterly-results.gif' }
			]
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
				{ type: 'file', name: 'cat-roomba.gif' },
			]
		},
		{ type: 'file', name: 'TODO.md' }
	];
</script>

<h1>MIO</h1>


<h2>Folder System Decks</h2>
<Folder name="Home" files={root} expanded/>





<h2>My Decks</h2>
<DeckList />
<hr />
<a href="/create-deck"><button>Create Deck</button></a>
<br />
<hr />
<a href="/demo">Svelte Examples</a>
