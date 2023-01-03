<script lang="ts">
	import Greet from '../lib/Greet.svelte';
	import DeckList from '../lib/DeckList.svelte';
	import TestPostButton from '$lib/TestPostButton.svelte';
	import CreateDeckFromCsvButton from '$lib/CreateDeckFromCSVButton.svelte';
</script>

<h1>Welcome to MI/O2</h1>
<em>Memory Input Output 2</em>
<h2>My Decks</h2>
<br />
<DeckList />

<h2>Create Deck</h2>
<button>Create Deck</button>
<button>Create Deck From CSV</button>

<Greet />

<TestPostButton />

<br />
<hr />
<br />
<CreateDeckFromCsvButton />
