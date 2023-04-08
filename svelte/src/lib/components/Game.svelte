<script lang="ts">
    import { onMount } from "svelte";

    // https://natclark.com/tutorials/svelte-get-current-url/
    let url: URL | null = null;
    let socketUrl = "";
    let joinToken = "";

    onMount(() => {
        url = new URL(window.location.href);
        const socketProtocol = url.protocol === "https:" ? "wss://" : "ws://";
        socketUrl = `${socketProtocol}${url.host}/ws`;

        joinToken = url.searchParams.get("token") || "";
        if (joinToken) {
            playerName = url.searchParams.get("name") || "";
            joinGame();
        }
    });

    interface GameState {
        turn: Team;
        winner: "Draw" | { Win: Team } | null;
        players: Player[];
        board: Square[];
        chat: ChatMessage[];
    }

    interface Player {
        team: Team;
        name: string;
        wins: number;
    }

    type Team = "X" | "O";
    type Square = " " | "X" | "O";

    interface ChatMessage {
        id: number;
        source: ChatMessageSource | "System";
        text: string;
    }

    type ChatMessageSource = PlayerSource | "System";
    interface PlayerSource {
        Player: Team;
    }

    let playerName = "";
    let inGame = false;
    let enoughPlayers = false;
    let myTeam: Team = "X";

    let gameState: GameState = {
        turn: "X",
        winner: null,
        players: [],
        board: [" ", " ", " ", " ", " ", " ", " ", " ", " "],
        chat: [],
    };
    function getPlayer(gameState: GameState, team: Team) {
        return gameState.players.find((p) => p.team === team);
    }

    let ws: WebSocket | null = null;

    function joinGame() {
        if (inGame) {
            return;
        }
        if (!socketUrl) {
            throw new Error("socketUrl not set");
        }

        console.log(
            "Joining game with player name:",
            playerName,
            "and join token:",
            joinToken
        );

        const url = new URL(socketUrl);
        url.searchParams.set("token", joinToken);
        url.searchParams.set("name", playerName);

        ws = new WebSocket(url.href);

        ws.onopen = () => {
            chatMessage = "";
            inGame = true;
        };

        ws.onmessage = (rawMsg) => {
            console.debug("Got Msg From Server:", rawMsg);
            const json = JSON.parse(rawMsg.data);
            console.debug("json", json);
            const type = Object.keys(json)[0].toString();
            const data = json[type];
            // console.debug("type", type, "data", data);

            if (type === "JoinedGame") {
                const { token, team, state } = data;
                joinToken = token as string;
                gameState = state as GameState;
                myTeam = team as Team;
                enoughPlayers = gameState.players.length === 2;
            } else if (type === "GameState") {
                gameState = data as GameState;
                enoughPlayers = gameState.players.length === 2;
            } else if (type === "Error") {
                console.error("Error from server", data);
                window.alert(data);
            } else {
                console.error("Unknown message type", type);
            }
        };

        ws.onclose = () => {
            inGame = false;
            joinToken = "";
            console.log("disconnected by server");
        };

        ws.onerror = (err) => {
            console.error("error", err);
        };
    }

    function leaveGame() {
        console.log("Leaving game");
        if (ws) {
            ws.close();
        }
    }

    let chatMessage = "";
    function isChatMessageValid(chatMessage: string): boolean {
        return (chatMessage || "").replace(/^\s+|\s+$/gm, "").length > 0;
    }

    function sendChatMessage() {
        if (!ws) {
            return;
        }
        if (!isChatMessageValid(chatMessage)) {
            return;
        }
        ws.send(JSON.stringify({ ChatMsg: { text: chatMessage } }));
        chatMessage = "";
    }

    function sendMove(space: number) {
        if (!ws) {
            return;
        }
        if (!enoughPlayers) {
            console.warn("not enough players to play");
            return;
        }
        if (gameState.turn !== myTeam) {
            console.warn("not my turn");
            return;
        }
        if (gameState.winner) {
            console.warn("game is over");
            return;
        }
        ws.send(JSON.stringify({ Move: { space } }));
    }

    function rematch() {
        if (!ws) {
            return;
        }
        if (!gameState.winner) {
            console.warn("game is not over");
            return;
        }
        ws.send(JSON.stringify({ Rematch: {} }));
    }
</script>

<div id="menu">
    <form id="join-game-form" on:submit|preventDefault={joinGame}>
        <div class="row">
            <div class="column">
                <label for="player-name">Player Name</label>
                <input
                    type="text"
                    id="player-name"
                    name="name"
                    placeholder="Player Name"
                    readonly={inGame}
                    bind:value={playerName}
                />
            </div>
            <div class="column">
                <label for="join-token"
                    >{inGame
                        ? "Code For Others to Join You"
                        : "Game Name"}</label
                >
                <input
                    type="text"
                    id="join-token"
                    name="token"
                    placeholder="Game Name (leave blank for random)"
                    readonly={inGame}
                    on:click={(e) => {
                        if (inGame) {
                            e.currentTarget.select();
                            navigator.clipboard.writeText(joinToken);
                        }
                    }}
                    bind:value={joinToken}
                />
            </div>
            {#if inGame}
                <div class="column">
                    <button
                        type="button"
                        on:click={leaveGame}
                        class="horizontal-submit"
                    >
                        Leave Game
                    </button>
                </div>
            {:else}
                <div class="column">
                    <button type="submit" class="horizontal-submit">
                        Join or Start Game
                    </button>
                </div>
            {/if}
        </div>
    </form>
</div>

{#if inGame}
    <div class="status">
        {#if !enoughPlayers}
            Waiting for opponent...
        {:else if gameState.winner}
            {#if gameState.winner === "Draw"}
                Draw!
            {:else if gameState.winner.Win === myTeam}
                You won!
            {:else}
                You lost!
            {/if}
        {:else if gameState.turn === myTeam}
            Your turn
        {:else}
            Opponent's turn
        {/if}
    </div>

    <div class="row">
        <div class="column">
            <div class="game-board">
                {#each gameState.board as square, i}
                    <button
                        type="button"
                        class="game-square {square}"
                        disabled={!enoughPlayers ||
                            gameState.winner !== null ||
                            gameState.turn !== myTeam ||
                            square !== " "}
                        on:click={() => sendMove(i)}
                    >
                        {square}
                    </button>
                {/each}
            </div>
        </div>

        <div class="column">
            <div id="chat">
                <h2>Chat</h2>
                <div class="chat-messages">
                    {#each gameState.chat as { id, source, text }}
                        <div class="chat-message" id={`chat-message-${id}`}>
                            {#if source === "System"}
                                <span class="chat-message-server">
                                    Server:
                                </span>
                            {:else}
                                <span class="chat-message-player">
                                    {getPlayer(gameState, source.Player)?.name} ({source.Player}):
                                </span>
                            {/if}
                            <span class="chat-message-text">{text}</span>
                        </div>
                    {/each}
                </div>
                <form on:submit|preventDefault={sendChatMessage}>
                    <div class="row">
                        <div class="column">
                            <input
                                type="text"
                                id="chat-msg"
                                name="msg"
                                placeholder="Message"
                                bind:value={chatMessage}
                            />
                        </div>
                        <div class="column">
                            <input
                                type="submit"
                                value="Send"
                                disabled={!inGame ||
                                    !isChatMessageValid(chatMessage)}
                            />
                        </div>
                        {#if gameState.winner}
                            <div class="column">
                                <button type="button" on:click={rematch}>
                                    Rematch
                                </button>
                            </div>
                        {/if}
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}
