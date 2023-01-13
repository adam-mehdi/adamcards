<script lang="ts">
	import CogButton from '$lib/CogButton.svelte';
	import { goto } from '$app/navigation';
	import {
		BaseDirectory,
		readTextFile,
		removeDir,
		renameFile,
		writeTextFile
	} from '@tauri-apps/api/fs';
	import { ask } from '@tauri-apps/api/dialog';
	import { sep } from '@tauri-apps/api/path';
	import toml from 'toml';

	export let deckName: string;
	export let deckDeadlineDate: string;
	export let deckDeadlineTime: string;
	export let readDeckEntires: () => void;

	const DECKS_DIR = 'decks';

	async function handleCogRoute(route: string) {
		try {
			// TODO: make the route the route for this deck
			await goto(route);
			return true;
		} catch {
			return false;
		}
	}

	async function handleDeleteDeck(deckName: string) {
		let re = await ask(
			`Are you sure you want to delete ${deckName}? This will delete all of ${deckName}'s cards.'`
		);

		if (re) {
			// delete
			let deckPath = DECKS_DIR + sep + deckName;
			deckPath.replace(/ \//g, '\n');
			try {
				removeDir(deckPath, { dir: BaseDirectory.AppData, recursive: true }).then(() => {
					readDeckEntires();
					return true;
				});

				return true;
			} catch (err) {
				console.error(err);
				return false;
			}
		} else {
			return false;
		}
	}

	let cogMenuOptions: gearMenuOption[] = [
		{ name: 'Edit Deck', action: async () => await handleCogRoute('/') }, // route to review
		{ name: 'Edit Name', action: handleEditName },
		{ name: 'Edit Deadline', action: handleEditDeadline },
		{ name: 'Delete Deck', action: async () => await handleDeleteDeck(deckName) }
	];

	interface gearMenuOption {
		name: String;
		action: () => Promise<boolean>;
	}

	let deckNameEditable = false;
	let deckDeadlineEditable = false;

	async function handleEditDeadline() {
		deckDeadlineEditable = true;
		return true;
	}

	async function handleEditName() {
		deckNameEditable = true;
		return true;
	}

	let oldName = deckName;
	let formName = deckName;
	function handleNameFormSubmit() {
		// TODO
		deckName = formName;
		let pathToOldName = DECKS_DIR + sep + oldName;
		pathToOldName.replace(/ \//g, '\n');
		let pathToNewName = DECKS_DIR + sep + deckName;
		pathToNewName.replace(/ \//g, '\n');

		try {
			renameFile(pathToOldName, pathToNewName, { dir: BaseDirectory.AppData });
		} catch (err) {
			console.error(`Error renaming deck: ${err}`);
		}
		deckNameEditable = false;
	}

	function getNumBoxes(deadline: string) {
		return 1;
	}

	async function handleDeadlineFormSubmit() {
		const configPath = DECKS_DIR + sep + deckName + sep + 'config.toml';
		configPath.replace(/ \//g, '\n');

		let rfc3339String = `${deckDeadlineDate}T${deckDeadlineTime}Z`;

		let num_boxes = getNumBoxes(rfc3339String);

		let configString = `num_boxes = ${num_boxes}\ndeadline = "${rfc3339String}"`;

		await writeTextFile(configPath, configString, {
			dir: BaseDirectory.AppData
		});

		deckDeadlineEditable = false;
	}
</script>

<div class="deck" style:margin-left={`${deckName.split('~~').length}em`}>
	<span class="left">
		{#if deckNameEditable}
			<form on:submit={handleNameFormSubmit}>
				<input type="text" bind:value={formName} autofocus />
			</form>
		{:else}
			{deckName.split('~~').pop()}
		{/if}
	</span>
	<span class="right">
		{#if deckDeadlineEditable}
			<form on:submit={handleDeadlineFormSubmit}>
				<input type="date" bind:value={deckDeadlineDate} />
				<input type="time" bind:value={deckDeadlineTime} />
				<button type="submit">Save</button>
			</form>
		{:else}
			<em>{`${deckDeadlineDate} ${deckDeadlineTime}`}</em>
		{/if}

		<button>Review</button>
		<CogButton options={cogMenuOptions} />
	</span>
</div>

<style>
	.deck {
		/* background-color: blueviolet; */
		width: full;
		display: flex;
		justify-content: space-between;
		margin-bottom: 0.5em;
	}
	.left {
		/* background-color: blue; */
		min-width: fit-content;
	}

	.right {
		display: flex;
		justify-content: space-between;
		/* background-color: red; */
		width: 100%;
		max-width: 300px;
	}
</style>
