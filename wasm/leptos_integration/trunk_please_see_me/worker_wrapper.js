const worker = new Worker('./trunk_please_see_me/worker.js', { type: 'module' });
let askQueue = Promise.resolve();
let ready = false;

// Listen for ready *once*
worker.onmessage = function(e) {
  if (e.data[0] === 'ready') {
    ready = true;
    console.log('Worker is ready');
  }
};

// Global function – must exist immediately, returns a Promise
window.javascript_im_begging_you = function(msg) {
  askQueue = askQueue.then(function() {
    return new Promise(function(resolve) {
      function trySend() {
        if (ready) {
          worker.onmessage = function(e) { resolve(e.data); };
          worker.postMessage(msg);
        } else {
          setTimeout(trySend, 10);
        }
      }
      trySend();
    });
  });
  return askQueue;
};

// Optional: auto‑initialise
window.javascript_im_begging_you(['initialize', 'leptos_db']).then(function(res) {
  console.log('DB initialized:', res);
});
