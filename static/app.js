function mustGetById(id) {
    const elt = document.getElementById(id);
    if (!elt) {
        throw new Error(`No element with id ${id}`);
    }
    return elt;
}

function showTimedAlert(msg) {
    const alert = mustGetById('alert');
    alert.innerText = msg.toString();
    alert.style.display = "block";
    setTimeout(() => {
        alert.style.display = "none";
    }, 10000);
}

function joinGame() {
    const joinTokenField = mustGetById('join-token');
    const joinToken = joinTokenField.value.toString();
    const joinGameForm = mustGetById('join-game-form');

    const gameArea = mustGetById('game-area');
    const chatMessages = mustGetById('chat-messages');
    const chatForm = mustGetById('chat-form');
    const chatMsgField = mustGetById('chat-msg');
    const leaveGameBtn = mustGetById('leave-game');

    const appendChatMessage = (id, msg) => {
        const elt = document.createElement('div');
        elt.id = "chat-msg-" + id;
        elt.className = "chat-msg";
        elt.appendChild(document.createTextNode(msg.toString()));
        chatMessages.appendChild(elt);
    }

    const updateView = (state) => {
        for (let [id, msg] of state.chat) {
            if (!document.getElementById("chat-msg-" + id)) {
                appendChatMessage(id, msg);
            }
        }
    }

    const ws = new WebSocket("ws://localhost:3000/ws?token=" + encodeURIComponent(joinToken));
    let connected = false;

    chatForm.addEventListener('submit', (e) => {
        e.preventDefault();
        if (!connected) {
            return;
        }
        const msg = chatMsgField.value;
        ws.send(msg);
        chatMsgField.value = "";
    });

    leaveGameBtn.addEventListener('click', (e) => {
        ws.close();
    });

    ws.onopen = () => {
        connected = true;
        showTimedAlert("Successfully joined game!");
        chatMsgField.value = "";
        joinGameForm.classList.add("hidden");
        gameArea.classList.remove("hidden");
    };

    ws.onmessage = (msg) => {
        console.debug("msg", msg);
        const state = JSON.parse(msg.data);
        updateView(state);
    }

    ws.onclose = () => {
        connected = false;
        console.debug("disconnected");
        joinTokenField.value = "";
        joinGameForm.classList.remove("hidden");
        gameArea.classList.add("hidden");
        showTimedAlert("Disconnected from game.");
    }

    ws.onerror = (err) => {
        console.error("err", err);
        appendChatMessage("err", "error");
    }
}

export default function app() {
    document.addEventListener('DOMContentLoaded', () => {
        mustGetById('join-game-form').addEventListener('submit', (e) => {
            e.preventDefault();
            joinGame();
        });
    });
}
