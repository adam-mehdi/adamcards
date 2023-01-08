<script lang="ts">
	export let name: string;
	$: file_type = name.slice(name.lastIndexOf('.') + 1);
</script>

<span>{name}</span>

<style>
	span {
		padding: 0 0 0 10px;
		background: 0 0.1em no-repeat;
		background-size: 1em 1em;
	}
</style>
