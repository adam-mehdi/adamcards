<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';

	interface DeckEntry {
		id: number;
		name: String;
		deadline_string: String;
	}

	let deckEntries: DeckEntry[] = [];

	async function getDecks() {
		deckEntries = await invoke('get_decks');
		console.log(deckEntries);
	}

	getDecks();
</script>

<table>
	<tr>
		<th>Name</th>
		<th>Deadline</th>
	</tr>
	{#each deckEntries as entry (entry.id)}
		<tr>
			<td>{entry.name}</td>
			<td>{entry.deadline_string}</td>
			<td><a href="/{entry.id}/review"><button>Edit</button></a></td>
			<td><a href="/{entry.id}/review"><button>Review</button></a></td>
		</tr>
	{/each}
</table>
