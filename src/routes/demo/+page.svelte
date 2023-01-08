<script lang="ts">
	import Greet from '$lib/illustrations/Greet.svelte';
	import DeckList from '$lib/illustrations/DeckList.svelte';
	import TestPostButton from '$lib/illustrations/TestPostButton.svelte';
	import CreateDeckFromCsvButton from '$lib/illustrations/CreateDeckFromCSVButton.svelte';
	import EditableCard from '$lib/illustrations/EditableCard.svelte';
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
