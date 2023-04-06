<script lang="ts">
    import Game from "../lib/components/Game.svelte";

    export let data: { socketUrl: string };
</script>

<div class="container">
    <h1>Tic Tac Toe</h1>
    <Game socketUrl={data.socketUrl} />
</div>
