import { createSignal, Match, Switch } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { Hello } from "./ipc/Hello"; // generated from shared::Hello
import "./App.css";

type Status = { kind: "loading" } | { kind: "ok" } | { kind: "error"; message: string };

function App() {
  const [status, setStatus] = createSignal<Status>({ kind: "loading" });

  // Startup checks: both must succeed. Nothing is logged unless one fails.
  Promise.all([invoke<Hello>("get_hello"), invoke("check_health")])
    .then(() => setStatus({ kind: "ok" }))
    .catch((err) => {
      console.error("startup check failed:", err);
      setStatus({ kind: "error", message: String(err) });
    });

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
    </main>
  );
}

export default App;
