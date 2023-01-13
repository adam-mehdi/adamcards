<script lang="ts">
	// This files handles:
	// - rendering list of decks
	// - editing deck names
	// - editing deck deadlines
	// - creating a new deck and giving it a name and deadline
	// - deleting a deck
	// - navigating to:
	//   - Editing the deck
	//   - Reviewing the Deck

	import type { DeckFolderInfo, DeckInfo } from '$lib/types/DecksTypes';
	import DeckFolder from '$lib/DeckFolder.svelte';
	import Deck from '$lib/Deck.svelte';
	import { dndzone } from 'svelte-dnd-action';
	import {
		writeTextFile,
		BaseDirectory,
		exists,
		readTextFile,
		createDir
	} from '@tauri-apps/api/fs';
	import { onMount } from 'svelte';

	const ROOT_PATH = 'root.json';

	const handleMount = async () => {
		try {
			rootExists = await exists(ROOT_PATH, { dir: BaseDirectory.AppData });
		} catch (err) {
			console.error(err);
			rootExists = false;
			console.log('One');
		}

		if (!rootExists) {
			let appDataDirExists: boolean | 'neither';
			try {
				appDataDirExists = await exists('', { dir: BaseDirectory.AppData });
				if (!appDataDirExists) {
					await createDir('', { dir: BaseDirectory.AppData });
				}
			} catch (err) {
				console.error(err);
			}

			try {
				await writeTextFile(ROOT_PATH, JSON.stringify(defaultRoot), { dir: BaseDirectory.AppData });
			} catch (err) {
				console.error(err);
				console.log('Two');
			}
		} else {
			try {
				const rootJSON = await readTextFile(ROOT_PATH, { dir: BaseDirectory.AppData });
				root = JSON.parse(rootJSON);
			} catch (err) {
				console.error(err);

				console.log('Three');
			}
		}
	};

	let defaultRoot: DeckFolderInfo = {
		id: 'ROOTDECKID',
		type: 'folder',
		name: 'Decks',
		contents: [{ id: 'EXAMPLEID', type: 'deck', name: 'Example Deck', deadline: null }],
		open: true
	};

	let rootExists = false;
	let root: DeckFolderInfo | null = defaultRoot;

	onMount(() => {
		handleMount();
	});

	// when root is updated at all, set a timeout, then write to the folders/root.json file
	$: {
		if (rootExists && root != null) {
			try {
				writeTextFile(ROOT_PATH, JSON.stringify(root), { dir: BaseDirectory.AppData });
				console.log(root);
			} catch (err) {
				console.error(err);
			}
		}
	}

	function handleCreateNewDeck() {
		// create new deck
		if (root) {
			let newDeck: DeckInfo = {
				id: self.crypto.randomUUID(), // TODO, assign it a random value...
				type: 'deck',
				name: 'New Deck',
				deadline: null
			};
			root?.contents?.push(newDeck);
			root = root;
		}
	}

	function handleCreateNewFolder() {
		// create new folder
		if (root) {
			let newFolder: DeckFolderInfo = {
				id: self.crypto.randomUUID(), // TODO, assign it a random value...
				type: 'folder',
				name: 'New Folder',
				contents: null,
				open: false
			};
			root?.contents?.push(newFolder);
			root = root;
		}
	}
</script>

{#if root}
	<div class="decks-container">
		<div><DeckFolder bind:folder={root} handleOwnDelte={null} /></div>
		<div class="add-buttons-container">
			<button class="add-button" on:click={handleCreateNewFolder} disabled={!root}>
				Add Folder
			</button>
			<button class="add-button" on:click={handleCreateNewDeck} disabled={!root}> Add Deck </button>
		</div>
	</div>
{:else}
	<div>No Folders Or Decks</div>
{/if}

<style>
	.decks-container {
		/* background-color: red; */
		height: 100%;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
	}
	div {
		list-style-type: none;
		margin: 0px;
		cursor: default;
	}
	.add-buttons-container {
		/* background-color: blue; */
		width: full;
		display: flex;
		justify-content: center;
	}
	.add-button {
		margin: 0px 1em 0px 1em;
	}
</style>
