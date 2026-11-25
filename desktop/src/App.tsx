import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { Hello } from "./ipc/Hello"; // generated from shared::Hello
import "./App.css";

function App() {
  const [message, setMessage] = createSignal("loading...");

  invoke<Hello>("get_hello")
    .then((hello) => setMessage(hello.message))
    .catch((err) => setMessage(`error: ${err}`));

  return (
    <main class="container">
      <h1>{message()}</h1>
    </main>
  );
}

export default App;