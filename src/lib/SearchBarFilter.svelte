<script lang="ts">
	export let prompt: string;
</script>

<div id="search-input-cont">
	<input
		type="text"
		id="search-field"
		placeholder="filter"
		autocomplete="off"
		bind:value={prompt}
		on:input
	/>
</div>

<style>
	#search-input-cont {
		display: flex;
		align-items: center;
	}

	#search-field {
		width: 100%;
		height: 80%;
		font-size: 1.3rem;
		border: 1px solid gray;
		border-radius: 5px;
		padding: 8px;
		margin: 0 10px 0;
	}
</style>
