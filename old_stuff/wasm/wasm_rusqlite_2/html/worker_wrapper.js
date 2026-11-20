// worker_wrapper.js – thin queue/transport for the worker.
// The caller is responsible for serializing Rust input structs into Uint8Array
// before calling ask(). The wrapper does not inspect or parse command payloads.
import { GiveMeSahpool, DoesSomebodyElseWantSahpool } from './share_sahpool.js';

const worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });

let askQueue = Promise.resolve();
let readyResolve = null;
let pendingResolve = null;
let hasConn = false;

const readyPromise = new Promise((resolve) => {
  readyResolve = resolve;
});

worker.onmessage = (e) => {
  const type = e.data[0];

  if (type === 'ready') {
    if (readyResolve) {
      readyResolve();
      readyResolve = null;
    }
    return;
  }

  if (type === 'want_conn') {
    GiveMeSahpool();
    hasConn = false;
    return;
  }

  if (pendingResolve) {
    // A response from any DB command means the connection is active.
    // close_conn explicitly clears it; errors may also mean no connection,
    // but for simplicity we keep previous behavior.
    if (type === 'close_conn') {
      hasConn = false;
    } else if (type !== 'error') {
      hasConn = true;
    }

    pendingResolve(e.data);
    pendingResolve = null;
  }
};

function ask(msg) {
  askQueue = askQueue
    .then(() => readyPromise)
    .then(
      () =>
        new Promise((resolve) => {
          pendingResolve = resolve;
          worker.postMessage(msg);
        })
    );

  return askQueue;
}

// Keep the old global API working.
window.javascript_im_begging_you = ask;

// Auto-initialize once the worker is ready.
// The initialize command remains special: the worker expects a string DB name.
window.javascript_im_begging_you(['initialize', 'leptos_db']).then((res) => {
  console.log('DB initialized:', res);
});

// Poll for other tabs/pages wanting the SAH pool, and close if needed.
setInterval(async () => {
  if (!hasConn) return;

  if (DoesSomebodyElseWantSahpool()) {
    console.log('[page] attempting to give up conn');
    await ask(['close_conn']);
    hasConn = false;
  }
}, 500);
