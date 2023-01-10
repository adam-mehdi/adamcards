<script lang="ts">
	import settingsGear from '$lib/images/settings-gear-black.png';
	export let open: boolean;
</script>

<button on:click|stopPropagation={() => (open = !open)} class="folder-settings-button">
	<img class="gear" src={settingsGear} alt="setting gear" />
</button>

<style>
	.gear {
		height: 1em;
		width: 1em;
	}
	.gear:hover {
		height: 1.2em;
		width: 1.2em;
	}
	.folder-settings-button {
		height: 100%;
		width: 2em;
		border: none;
		display: flex;
		justify-content: center;
		align-items: center;
		background-color: transparent;
		border-radius: 100px;
	}
</style>
