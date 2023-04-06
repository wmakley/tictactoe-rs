<script lang="ts">
    import { onMount } from "svelte";
    import Game from "../lib/components/Game.svelte";

    // https://natclark.com/tutorials/svelte-get-current-url/
    let href = "";
    onMount(() => (href = window.location.href));
    let socketUrl = "";
    $: {
        if (href) {
            const url = new URL(href);
            const socketProtocol =
                url.protocol === "https:" ? "wss://" : "ws://";
            socketUrl = `${socketProtocol}${url.host}/ws`;
            // console.debug("url", url, "host", url.host, "socketUrl", socketUrl);
        }
    }
</script>

<div class="container">
    <h1>Tic Tac Toe</h1>
    <Game {socketUrl} />
</div>
