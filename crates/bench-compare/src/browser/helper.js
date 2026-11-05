// Passes the measuring thread's `/begin` and `/end` to the driver, so the measuring thread runs
// none of the code of a request.

'use strict';

importScripts('/shared.js');

const call = (path) => {
  const request = new XMLHttpRequest();
  request.open('POST', path, false);
  request.send(null);
  if (request.status !== 200) {
    throw new Error(`${path}: HTTP ${request.status}: ${request.responseText}`);
  }
  return request.responseText;
};

onmessage = (event) => {
  const { state, counts } = views(event.data);
  // The last state that this worker stored or saw: it sleeps while the state stays there.
  let seen = IDLE;
  for (;;) {
    Atomics.wait(state, 0, seen);
    const command = Atomics.load(state, 0);
    if (command === seen) {
      continue;
    }
    let done = DONE;
    try {
      if (command === BEGIN) {
        call('/begin');
      } else if (command === END) {
        counts.set(call('/end').split(' ').map(BigInt));
      } else {
        throw new Error(`the command ${command}`);
      }
    } catch (error) {
      done = FAILED;
      const request = new XMLHttpRequest();
      request.open('POST', '/error', false);
      request.send(String(error && error.stack ? error.stack : error));
    }
    Atomics.store(state, 0, done);
    Atomics.notify(state, 0);
    seen = done;
  }
};
