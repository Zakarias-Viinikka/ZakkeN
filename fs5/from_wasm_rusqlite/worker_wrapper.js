import { GiveMeSahpool, DoesSomebodyElseWantSahpool } from './share_sahpool.js';

const worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });

let askQueue = Promise.resolve();
let readyResolve = null;
let pendingResolve = null;
let hasConn = false;

const readyPromise = new Promise(resolve => {
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
    if (type === 'close_conn') {
      hasConn = false;
    } else {
      hasConn = true;
    }

    pendingResolve(e.data);
    pendingResolve = null;
  }
};

function ask(msg) {
  askQueue = askQueue
    .then(() => readyPromise)
    .then(() => new Promise(resolve => {
      pendingResolve = resolve;
      worker.postMessage(msg);
    }));

  return askQueue;
}

// Keep the old global API working.
window.javascript_im_begging_you = ask;

// Auto-initialize once the worker is ready.
window.javascript_im_begging_you(['initialize', 'leptos_db']).then(res => {
  console.log('DB initialized:', res);
});

setInterval(async () => {
  if (!hasConn) return;

  if (DoesSomebodyElseWantSahpool()) {
    console.log('[page] attempting to give up conn');
    await ask(["close_conn"]);
    hasConn = false;
  }
}, 500);
