/* @refresh reload */
import { render } from "@solidjs/web";
import App from "./App";

// Entry point: mounts <App /> into the #root element in index.html
render(() => <App />, document.getElementById("root") as HTMLElement);
