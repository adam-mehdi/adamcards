<script lang="ts">
	import settingsGear from '$lib/images/settings-gear-black.png';
	import { clickOutside } from '$lib/actions/click_outside';

	let open: boolean = false;

	interface gearMenuOption {
		name: String;
		action: () => Promise<boolean>;
	}

	export let options: gearMenuOption[];

	async function handleOptionClick(fn: () => Promise<boolean>) {
		fn().then((exit) => {
			if (exit) open = false;
		});
	}
</script>

<button on:click|stopPropagation={() => (open = !open)} class="cog-menu-button">
	<img class="gear" src={settingsGear} alt="setting gear" />
	{#if open}
		<div
			class="cog-menu-tray"
			use:clickOutside={open}
			on:outclick={() => {
				open = !open;
			}}
		>
			{#each options as option}
				<button
					class="cog-menu-item"
					on:click|stopPropagation={async () => await handleOptionClick(option.action)}
					>{option.name}</button
				>
			{/each}
		</div>
	{/if}
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

	.cog-menu-item {
		margin: 2px;
	}

	.cog-menu-button {
		position: relative;
		height: 100%;
		width: 2em;
		border: none;
		display: flex;
		justify-content: center;
		align-items: center;
		background-color: transparent;
		border-radius: 100px;
	}

	.cog-menu-tray {
		z-index: 10;
		position: absolute;
		top: 2em;
		right: 0px;
		width: 200px;
		border-radius: 5px;
		padding: 1em;
		background-color: gainsboro;
		display: flex;
		flex-direction: column;
	}
</style>
