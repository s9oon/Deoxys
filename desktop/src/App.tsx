import { createSignal, For, Match, onCleanup, Switch } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { Hello } from "./ipc/Hello"; // generated from shared::Hello
import type { ClientMsg } from "./ipc/ClientMsg"; // generated from shared::ClientMsg
import type { ServerMsg } from "./ipc/ServerMsg"; // generated from shared::ServerMsg
import "./App.css";

// Result of the startup checks; drives the Ok!/error banner
type Status = { kind: "loading" } | { kind: "ok" } | { kind: "error"; message: string };

// Backend WebSocket address; set VITE_WS_URL at build time to point elsewhere
const WS_URL = import.meta.env.VITE_WS_URL ?? "ws://localhost:3000/ws";

function App() {
  const [status, setStatus] = createSignal<Status>({ kind: "loading" }); // startup check result
  const [connected, setConnected] = createSignal(false); // is the WebSocket open
  const [draft, setDraft] = createSignal(""); // text currently in the input box
  const [messages, setMessages] = createSignal<string[]>([]); // chat log, oldest first

  // Startup checks: both must succeed. Nothing is logged unless one fails.
  Promise.all([invoke<Hello>("get_hello"), invoke("check_health")])
    .then(() => setStatus({ kind: "ok" }))
    .catch((err) => {
      console.error("startup check failed:", err);
      setStatus({ kind: "error", message: String(err) });
    });

  // One WebSocket for the app's lifetime; every reply gets appended to the list.
  const socket = new WebSocket(WS_URL);
  socket.onopen = () => setConnected(true);
  socket.onclose = () => setConnected(false);
  socket.onmessage = (event) => {
    const msg = JSON.parse(event.data) as ServerMsg; // trusts the backend to send a valid ServerMsg
    if (msg.type === "Chat") {
      setMessages((prev) => [...prev, msg.text]);
    }
  };
  onCleanup(() => socket.close()); // close the socket if the component is removed

  // Sends the draft as a ClientMsg::Chat, then clears the input
  const send = (e: SubmitEvent) => {
    e.preventDefault(); // stop the form from reloading the page
    const text = draft().trim();
    if (!text || socket.readyState !== WebSocket.OPEN) return; // nothing to send, or not connected
    const msg: ClientMsg = { type: "Chat", text };
    socket.send(JSON.stringify(msg));
    setDraft("");
  };

  return (
    <main class="container">
      {/* Startup check banner: nothing shows while loading */}
      <Switch>
        <Match when={status().kind === "ok"}>
          <h1>Ok!</h1>
        </Match>
        <Match when={status().kind === "error"}>
          <p class="error">{(status() as { message: string }).message}</p>
        </Match>
      </Switch>

      {/* Chat: connection status, input form, and the message log */}
      <section class="chat">
        <p class="ws-status">{connected() ? "WebSocket connected" : "WebSocket disconnected"}</p>
        <form class="chat-form" onSubmit={send}>
          <input
            value={draft()}
            onInput={(e) => setDraft(e.currentTarget.value)}
            placeholder="Type a message and press Enter"
          />
          <button type="submit" disabled={!connected()}>
            Send
          </button>
        </form>
        <ul class="chat-log">
          <For each={messages()}>{(text) => <li>{text}</li>}</For>
        </ul>
      </section>
    </main>
  );
}

export default App;
