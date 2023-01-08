<script lang="ts">
	import File from './File.svelte';
	import KaTeXRenderer from './KaTeXRenderer.svelte';
	import settingsGear from '$lib/images/settings-gear-black.png';

	export let expanded = false;
	export let name = '';

	type FileSystemObject = {
		type: 'folder' | 'file';
		name: string;
		files?: FileSystemObject[];
	};
	export let files: FileSystemObject[];

	function toggle() {
		expanded = !expanded;
	}

	let settingsTrayOpen: boolean = true;
	function toggleSettingsTray() {
		settingsTrayOpen = !settingsTrayOpen;
	}
</script>

<span class="folder" class:expanded on:click={toggle} on:keypress={toggle}
	><span>{name}</span><span
		><img
			class="gear"
			on:keydown|stopPropagation={toggleSettingsTray}
			on:click|stopPropagation={toggleSettingsTray}
			src={settingsGear}
			alt="setting gear"
		/></span
	>
</span>

{#if expanded}
	<ul>
		{#each files as file}
			<li>
				{#if file.type === 'folder'}
					<svelte:self {...file} />
				{:else}
					<File {...file} />
				{/if}
			</li>
		{/each}
	</ul>
{/if}

<style>
	.folder {
		display: flex;
		justify-content: space-between;
		width: 100%;
		/* padding: 2px 2px 2px 1.5em; */
		/* background: url(/folder.svg) 0 0.1em no-repeat; */
		padding: 2px 10px;
		background: 0 0.1em no-repeat;
		background-size: 1em 1em;
		font-weight: bold;
		cursor: pointer;
		margin: 1px;
		border-radius: 2px;
	}

	.folder:hover {
		background-color: #eee;
	}

	/* .expanded {
		background-image: url(/folder-open.svg);
	} */

	ul {
		padding: 0.2em 0 0 0.5em;
		margin: 0 0 0 0.5em;
		list-style: none;
		border-left: 1px solid #eee;
	}

	li {
		padding: 0.2em 0;
	}

	.gear {
		height: 1em;
		width: 1em;
	}
</style>
