import init, { LiveForever } from '../pkg/part2.js';
await init();
const state = LiveForever.new("my data");

self.onmessage = (e) => {
  const msg = e.data;
  if (msg[0] === "get data") {
    self.postMessage(state.get_data());
  } else if (msg[0] === "change data") {
    state.change_data(msg[1]);          // pass the new value
    self.postMessage("changed");
  } else {
    self.postMessage("unknown");
  }
};

console.log("loaded worker js");

self.postMessage(["ready"]);
