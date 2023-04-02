<script lang="ts">
    interface GameState {
        players: Player[];
        board: Square[];
        chat: ChatMessage[];
    }

    interface Player {
        id: number;
        name: string;
        team: Team;
    }

    type Team = "X" | "O";
    type Square = " " | "X" | "O";

    interface ChatMessage {
        id: number;
        player: number;
        text: string;
    }

    let joinToken = "";
    let playerName = "";
    let inGame = false;

    let gameState: GameState = {
        players: [],
        board: [" ", " ", " ", " ", " ", " ", " ", " ", " "],
        chat: [],
    };

    let ws: WebSocket | null = null;

    function joinGame() {
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
            console.debug("msg", rawMsg);
            const json = JSON.parse(rawMsg.data);
            console.debug("json", json);
            const type = Object.keys(json)[0].toString();
            const data = json[type];
            console.debug("type", type, "data", data);

            if (type === "JoinedGame") {
                const { token, state } = data;
                joinToken = token;
                gameState = state;
            } else if (type === "GameState") {
                gameState = data;
            } else {
                console.error("Unknown message type", type);
            }
        };

        ws.onclose = () => {
            inGame = false;
            console.log("disconnected");
        };

        ws.onerror = (err) => {
            console.error("err", err);
        };
    }

    function leaveGame() {
        console.log("Leaving game");
        if (ws) {
            ws.close();
        }
    }

    let chatMessage = "";
    function sendChatMessage() {
        if (!ws) {
            return;
        }
        ws.send(JSON.stringify({ ChatMsg: { text: chatMessage } }));
        chatMessage = "";
    }
</script>

{#if !inGame}
    <div id="menu">
        <form id="join-game-form" on:submit|preventDefault={joinGame}>
            <div class="row">
                <div class="column">
                    <input
                        type="text"
                        id="player-name"
                        name="name"
                        placeholder="Player Name"
                        required
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
                <div class="column">
                    <input type="submit" value="Join Game" />
                </div>
            </div>
        </form>
    </div>
{/if}

{#if inGame}
    <div class="row" id="game-area">
        <div class="column">
            <div id="game-board">
                <div class="game-row">
                    <div class="square" />
                    <div class="square" />
                    <div class="square" />
                </div>
                <div class="game-row">
                    <div class="square" />
                    <div class="square" />
                    <div class="square" />
                </div>
                <div class="game-row">
                    <div class="square" />
                    <div class="square" />
                    <div class="square" />
                </div>
            </div>
        </div>

        <div class="column">
            <div id="chat">
                <h2>Chat</h2>
                <div class="chat-messages">
                    {#each gameState.chat as { id, player, text }}
                        <div class="chat-message" id={`chat-message-${id}`}>
                            <span class="chat-message-player">
                                {gameState.players[player].name}:
                            </span>
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
                            <input type="submit" value="Send" />
                            <button type="button" on:click={leaveGame}>
                                Leave Game
                            </button>
                        </div>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}
