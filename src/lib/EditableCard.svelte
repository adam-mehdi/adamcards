<script lang="ts">
	import { save } from '@tauri-apps/api/dialog';
	import katex from 'katex';
	import KaTeXRenderer from './KaTeXRenderer.svelte';

	let front = '';
	let back = '';
	let backendFront = '';
	let backendBack = '';
	let saved: boolean = false;
	let saving: boolean = false;
	let timeoutId: NodeJS.Timeout | null = null;
	let innerTimeoutId: NodeJS.Timeout | null = null;

	const handleChange = () => {
		timeoutId && clearTimeout(timeoutId);
		saved = false;
		timeoutId = setTimeout(onTimeout, 3000);
	};

	const onTimeout = () => {
		innerTimeoutId && clearTimeout(innerTimeoutId);
		saving = true;
		innerTimeoutId = setTimeout(onInnerTimeout, 1000);
	};

	const onInnerTimeout = () => {
		saved = true;
		backendFront = front;
		backendBack = back;
		saving = false;
	};
</script>

<hr />
<h1>EditableCard.Svelte</h1>
<div>
	<label>
		Front
		<input bind:value={front} on:input={handleChange} />
	</label>
	<label>
		Back
		<input bind:value={back} on:input={handleChange} />
	</label>
	<p>{saved ? 'Saved' : saving ? 'Saving...' : ''}</p>
</div>
<h3>Hypothetical Backend State</h3>
{backendFront}
{backendBack}
<p>Front: <KaTeXRenderer input={backendFront} /></p>
<p>Back: <KaTeXRenderer input={'TEST TEST'} /></p>
<hr />
