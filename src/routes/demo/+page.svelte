<script lang="ts">
	import Greet from '$lib/Greet.svelte';
	import DeckList from '$lib/DeckList.svelte';
	import TestPostButton from '$lib/TestPostButton.svelte';
	import CreateDeckFromCsvButton from '$lib/CreateDeckFromCSVButton.svelte';
	import EditableCard from '$lib/EditableCard.svelte';
	import KaTeXRenderer from '$lib/KaTeXRenderer.svelte';
</script>

<a href="/">Back to Home</a>
<hr />
<br />

<Greet />

<TestPostButton />
<br />
<hr />
<br />
<CreateDeckFromCsvButton />
<br />
<hr />
<br />

<br />
<KaTeXRenderer input={String.raw`Hello:`} />
<br />
<EditableCard />
