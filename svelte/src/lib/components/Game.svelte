<script lang="ts">
    interface GameState {
        turn: Team;
        winner: Team | null;
        players: Player[];
        board: Square[];
        chat: ChatMessage[];
    }

    interface Player {
        team: Team;
        name: string;
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
        Player: {
            team: Team;
        };
    }

    let joinToken = "";
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

        console.log(
            "Joining game with player name:",
            playerName,
            "and join token:",
            joinToken
        );

        ws = new WebSocket(
            "ws://localhost:3000/ws?token=" +
                encodeURIComponent(joinToken) +
                "&name=" +
                encodeURIComponent(playerName)
        );

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
    let isChatMessageValid = false;
    $: {
        isChatMessageValid =
            (chatMessage || "").replace(/^\s+|\s+$/gm, "").length > 0;
    }

    function sendChatMessage() {
        if (!ws) {
            return;
        }
        if (!isChatMessageValid) {
            return;
        }
        ws.send(JSON.stringify({ ChatMsg: { text: chatMessage } }));
        chatMessage = "";
    }

    function sendMove(space: number) {
        if (!ws) {
            return;
        }
        // if (!enoughPlayers) {
        //     console.warn("not enough players to play");
        //     return;
        // }
        ws.send(JSON.stringify({ Move: { space } }));
    }
</script>

<div id="menu">
    <form id="join-game-form" on:submit|preventDefault={joinGame}>
        <div class="row">
            <div class="column">
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
                <input
                    type="text"
                    id="join-token"
                    name="token"
                    placeholder="Join Token (Leave blank for new game)"
                    readonly={inGame}
                    bind:value={joinToken}
                />
            </div>
            {#if inGame}
                <div class="column">
                    <button type="button" on:click={leaveGame}>
                        Leave Game
                    </button>
                </div>
            {:else}
                <div class="column">
                    <input type="submit" value="Join Game" />
                </div>
            {/if}
        </div>
    </form>
</div>

{#if inGame}
    <div class="row">
        <div class="column">
            <div class="game-board">
                {#each gameState.board as square, i}
                    {#if i % 3 === 0}
                        <div class="clear" />
                    {/if}
                    <button
                        type="button"
                        class="square"
                        disabled={!enoughPlayers ||
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
                                    {gameState.players.filter(
                                        (p) => p.team === myTeam
                                    )[0].name} ({myTeam}):
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
                                disabled={!inGame || !isChatMessageValid}
                            />
                        </div>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}
