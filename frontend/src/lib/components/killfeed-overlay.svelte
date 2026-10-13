<script lang="ts">
  let { character }: Record<string, unknown> = $props();
</script>

<p>{JSON.stringify(character)}</p>
