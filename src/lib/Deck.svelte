<script lang="ts">
	import fsStore from '$lib/stores/fsStore'
	import SettingsTrayButton from './SettingsTrayButton.svelte';

	export let name: string;
	export let path: string;
	$: file_type = name.slice(name.lastIndexOf('.') + 1);

	

</script>

<div class="buttons flow-root">
	<span class="float-left">
		{name}
	</span>

	<!-- SettingsTray -->
	<SettingsTrayButton entryType="deck" path={path} />
	
</div>



<style>
	span {
		padding: 0 0 0 1.5em;
		background: url(/icons8-king-of-spades-50.png) 0 0.1em no-repeat;
		background-size: 1em 1em;
		background-size: 16px;
	}
	
</style>