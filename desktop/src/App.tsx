import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [message, setMessage] = createSignal("");

  // Calls the `greet` function in src-tauri/src/lib.rs
  invoke<string>("greet", { name: "World" }).then(setMessage);

  return (
    <main class="container">
      <h1>{message()}</h1>
    </main>
  );
}

export default App;
