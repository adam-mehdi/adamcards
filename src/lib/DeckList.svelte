<script lang="ts">
	// This is a temporary deck list so that the app is useable while I figure out how to deal with the drag and drop.

	import {
		readDir,
		BaseDirectory,
		exists,
		createDir,
		writeTextFile,
		type FileEntry,
		readTextFile
	} from '@tauri-apps/api/fs';
	import { onMount } from 'svelte';
	import Deck from './Deck.svelte';

	import toml from 'toml';
	import DeckListDeck from './DeckListDeck.svelte';
	import { sep } from '@tauri-apps/api/path';

	// DECKS_DIR must not have a leading "/" as this will cause tauri to interpret it as an absolute path relative to the C: directory (on windows)
	// unsure of behavior on mac, I imagine it would also make it relative to the system's root directory
	const DECKS_DIR = 'decks';

	let deckEntries: FileEntry[];

	let createDeckShown = false;

	onMount(async () => {
		await readDeckEntries();
	});

	async function readDeckEntries() {
		let decksDirExists = false;
		// check if decks directory exists
		try {
			decksDirExists = await exists(DECKS_DIR, { dir: BaseDirectory.AppData });
		} catch (err) {
			console.error(err);
		}

		// if the decks directory has been created
		if (decksDirExists) {
			console.log('deck dir exists');
			// try to read it's contents
			try {
				deckEntries = await readDir(DECKS_DIR, { dir: BaseDirectory.AppData });
				console.log('ENTRIES FROM 44');
				console.log(deckEntries);
				return;
				// processEntries(deckEntries);
				// console.log('decks:');
				// console.log(deckEntries);
			} catch (err) {
				console.error(err);
			}
			// if the decks directory doesn't exist, try to create it
		} else {
			try {
				// console.log('attempting to create decks dir');
				createDir(DECKS_DIR, { dir: BaseDirectory.AppData });
				// console.log('decks dir created');
			} catch (err) {
				console.error(err);
			}
		}
	}

	// assumes that deadline is well formatted
	function getNumBoxes(deadline: string): number {
		return 1;
	}

	// Writes a new deck to the file system
	// assumes that deadline is well formatted
	async function writeNewDeck(
		deckName: string,
		deadline: string
	): Promise<'created' | 'already exists' | 'error'> {
		const deckPath = DECKS_DIR + '/' + deckName;

		// check if the deck with that name already exists
		try {
			let deckExists = await exists(deckPath, { dir: BaseDirectory.AppData });
			if (deckExists) {
				return 'already exists';
			}
		} catch (err) {
			console.error(err);
			return 'error';
		}

		// create the deck's directory
		try {
			await createDir(deckPath, { dir: BaseDirectory.AppData });
			// create the cards.csv file (empty)
			await writeTextFile(deckPath + '/cards.csv', '', { dir: BaseDirectory.AppData });
			// create the config.toml

			let num_boxes = getNumBoxes(deadline);
			let configString = `num_boxes = ${num_boxes}\ndeadline = "${deadline}"`;

			await writeTextFile(deckPath + '/config.toml', configString, {
				dir: BaseDirectory.AppData
			});
			// create quotas.csv
			await writeTextFile(deckPath + '/quotas.csv', '', { dir: BaseDirectory.AppData });
		} catch (err) {
			console.error(err);
			return 'error';
		}
		readDeckEntries();
		return 'created';
	}

	let formDeckName = '';
	let formDeckDeadlineDate = '';
	let formDeckDeadlineTime = '14:30';

	function handleNewDeckSubmit(
		newDeckName: string,
		newDeckDeadlineDate: string,
		newDeckDeadlineTime: string
	) {
		// check that deck name is well formatted
		// construct rfc3339 string
		let rfc3339String = `${newDeckDeadlineDate}T${newDeckDeadlineTime}:00+00:00`;
		writeNewDeck(newDeckName, rfc3339String).then((re) => {
			if (re == 'created') {
				formDeckName = '';
				formDeckDeadlineDate = '';
				formDeckDeadlineTime = '';
			} else if (re === 'already exists') {
				alert(`A deck named ${newDeckName} already exists!`);
			} else if (re === 'error') {
				alert(`Unknown error in writeNewDeck`);
			}
		});
	}

	interface DeckInfo {
		deckName: string;
		deckDeadlineDate: string;
		deckDeadlineTime: string;
	}

	async function getDeadline(deckName: string): Promise<{ date: string; time: string }> {
		// read in the deck's config.toml based on the deck name
		try {
			const configPath = DECKS_DIR + sep + deckName + sep + 'config.toml';
			configPath.replace(/ \//g, '\n');

			let configText = await readTextFile(configPath, { dir: BaseDirectory.AppData });
			try {
				let config = toml.parse(configText);
				if (config.deadline != '') {
					let [date, time] = config.deadline.split('T');
					time = time.slice(0, 5);
					return { date: date, time: time };
				} else {
				}
				// console.log(config);
			} catch (err) {
				console.error(`ERROR PARSING TOML: ${err}`);
			}

			return { date: '', time: '' };
		} catch (err) {
			console.error(err);
			return { date: '', time: '' };
		}
	}

	async function getDecksInfo(deckEntries: FileEntry[]) {
		// let output: DeckInfo[] = [];
		if (deckEntries && deckEntries.length != 0) {
			decks = [];
			deckEntries.forEach(async (entry) => {
				if (entry.name) {
					let deadline = await getDeadline(entry.name);
					let outDeck = {
						deckName: entry.name,
						deckDeadlineDate: deadline.date,
						deckDeadlineTime: deadline.time
					};
					decks.push(outDeck);
					decks = decks;
				}
			});
		} else {
			decks = [];
		}
	}

	let decks: DeckInfo[] = [];

	const handleUpdate = async () => {
		await getDecksInfo(deckEntries);
	};

	$: {
		if (deckEntries) {
			handleUpdate();
		}
	}
</script>

<h3>Decks</h3>
{#if decks.length != 0}
	{#each decks as deck, i (deck.deckName.toLowerCase())}
		{#if deck.deckName.split('~~').length > 1}
			{#each deck.deckName.split('~~').slice(0, -1) as folder, j}
				{#if i < 1 || decks[i].deckName.split('~~')[j] != decks[i - 1].deckName.split('~~')[j]}
					<div style:margin-left={`${j + 1}em`}><strong>{folder}</strong></div>
				{/if}
			{/each}
		{/if}
		<DeckListDeck
			bind:deckName={deck.deckName}
			bind:deckDeadlineDate={deck.deckDeadlineDate}
			bind:deckDeadlineTime={deck.deckDeadlineTime}
			readDeckEntires={() => readDeckEntries()}
		/>
	{/each}
{:else}
	<p><em>No Decks</em></p>
{/if}

{#if createDeckShown}
	<div class="create-deck-form-area">
		<form
			on:submit|preventDefault={() =>
				handleNewDeckSubmit(formDeckName, formDeckDeadlineDate, formDeckDeadlineTime)}
		>
			<input type="text" bind:value={formDeckName} required autofocus />
			<input type="date" bind:value={formDeckDeadlineDate} required />
			<input type="time" bind:value={formDeckDeadlineTime} required />

			<button type="submit">Create Deck</button>
			<button
				on:click={() => {
					createDeckShown = !createDeckShown;
				}}>Cancel</button
			>
		</form>
	</div>
{:else}
	<button
		class="show-create-deck"
		on:click={() => {
			createDeckShown = !createDeckShown;
		}}>+</button
	>
{/if}

<style>
	.show-create-deck {
		margin-top: 3em;
		width: 100%;
	}
	.create-deck-form-area {
		margin-top: 3em;
		width: 100%;
		display: flex;
		justify-content: center;
	}
</style>
