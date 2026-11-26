import { createSignal, For, Match, onCleanup, Switch } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { Hello } from "./ipc/Hello"; // generated from shared::Hello
import type { ClientMsg } from "./ipc/ClientMsg"; // generated from shared::ClientMsg
import type { ServerMsg } from "./ipc/ServerMsg"; // generated from shared::ServerMsg
import "./App.css";

type Status = { kind: "loading" } | { kind: "ok" } | { kind: "error"; message: string };

const WS_URL = import.meta.env.VITE_WS_URL ?? "ws://localhost:3000/ws";

function App() {
  const [status, setStatus] = createSignal<Status>({ kind: "loading" });
  const [connected, setConnected] = createSignal(false);
  const [draft, setDraft] = createSignal("");
  const [messages, setMessages] = createSignal<string[]>([]);

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
    const msg = JSON.parse(event.data) as ServerMsg;
    if (msg.type === "Chat") {
      setMessages((prev) => [...prev, msg.text]);
    }
  };
  onCleanup(() => socket.close());

  const send = (e: SubmitEvent) => {
    e.preventDefault();
    const text = draft().trim();
    if (!text || socket.readyState !== WebSocket.OPEN) return;
    const msg: ClientMsg = { type: "Chat", text };
    socket.send(JSON.stringify(msg));
    setDraft("");
  };

  return (
    <main class="container">
      <Switch>
        <Match when={status().kind === "ok"}>
          <h1>Ok!</h1>
        </Match>
        <Match when={status().kind === "error"}>
          <p class="error">{(status() as { message: string }).message}</p>
        </Match>
      </Switch>

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
