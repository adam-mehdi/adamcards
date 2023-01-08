<script lang="ts">
	import DeckList from '$lib/illustrations/DeckList.svelte';
	import Folder from '$lib/Folder.svelte';
	import { exclude_internal_props } from 'svelte/internal';

	// modes: normal and editing
	// - normal: go to review when clicking on deck, expand/collapse on deck
	// - editing: edit when clicking on deck, create new deck when clicking folder

	type FileSystemObject = {
		type: 'folder' | 'file';
		name: string;
		files?: FileSystemObject[];
		expanded?: boolean;
	};

	// load contents of ~/data/decks (call to backend)

	let root: FileSystemObject[] = [
		{
			type: 'folder',
			name: 'Important work stuff',
			files: [{ type: 'file', name: 'quarterly-results.gif' }],
			expanded: true
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
					],
					expanded: false
				},
				{ type: 'file', name: 'cat-roomba.gif' }
			]
		},
		{ type: 'file', name: 'TODO.md' }
	];
	let deck_name = 'test';
</script>

<h1>MIO</h1>

<h2>Decks</h2>
<div class="folders">
	<Folder name="My Decks" files={root} expanded />
</div>

<h2>Test deck</h2>
<a href="/{deck_name}/edit"><button>Edit/Create</button></a>
<br>
<a href="/{deck_name}/review"><button>Review</button></a>


<style>
	.folders {
		width: 100vw;
		max-width: 600px;
	}
</style>
