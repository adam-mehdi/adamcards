<script lang="ts">
	import Greet from '../lib/Greet.svelte';
	import DeckList from '../lib/DeckList.svelte';
	import TestPostButton from '$lib/TestPostButton.svelte';
	import CreateDeckFromCsvButton from '$lib/CreateDeckFromCSVButton.svelte';
	import EditableCard from '$lib/EditableCard.svelte';
	import KaTeXRenderer from '$lib/KaTeXRenderer.svelte';
</script>

<h1>Welcome to MI/O2</h1>
<em>Memory Input Output 2</em>
<h2>My Decks</h2>
<DeckList />
<hr />
<a href="/create-deck"><button>Create Deck</button></a>
<br />
<hr />
<a href="/demo">Svelte Examples</a>
