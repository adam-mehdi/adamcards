<script lang="ts">
	export let name: string;
	export let editable: boolean;
</script>

<span class="folder-name-container">
	{#if editable}
		<input type="text" bind:value={name} />
	{:else}
		<span
			class="folder-name"
			on:keydown={() => (editable = true)}
			on:dblclick={() => (editable = true)}>{name}</span
		>
	{/if}
</span>

<style>
	.folder-name {
		font-weight: bold;
		cursor: text;
	}
</style>
