// Runs the bench module in a dedicated worker, the measuring thread, which the driver counts.
// `/begin` and `/end` go through the helper worker: the measuring thread sleeps in `Atomics.wait`
// while the helper's synchronous request runs and the driver reads the counters. Every other
// import is a synchronous request of its own.

'use strict';

importScripts('/shared.js');

const call = (path, body) => {
  const request = new XMLHttpRequest();
  request.open('POST', path, false);
  request.send(body === undefined ? null : body);
  if (request.status !== 200) {
    throw new Error(`${path}: HTTP ${request.status}: ${request.responseText}`);
  }
  return request.responseText;
};

// The driver finds the measuring thread by its CPU time while this loop spins.
const spin = (milliseconds) => {
  const start = performance.now();
  while (performance.now() - start < milliseconds) {
    // Spin.
  }
};

onmessage = async (event) => {
  const { state, counts } = views(event.data);
  // Hands `command` to the helper and sleeps until it is done.
  const ask = (command) => {
    Atomics.store(state, 0, command);
    Atomics.notify(state, 0);
    while (Atomics.load(state, 0) === command) {
      Atomics.wait(state, 0, command);
    }
    if (Atomics.load(state, 0) !== DONE) {
      throw new Error(`the helper failed ${command === BEGIN ? '/begin' : '/end'}`);
    }
  };
  try {
    const module = await WebAssembly.compile(await (await fetch('/bench.wasm')).arrayBuffer());
    let memory = null;
    const text = (address, length) =>
      new TextDecoder().decode(new Uint8Array(memory.buffer, address, length));
    const instance = await WebAssembly.instantiate(module, {
      bench: {
        begin: () => ask(BEGIN),
        end: (address) => {
          ask(END);
          new BigUint64Array(memory.buffer, address, counts.length).set(counts);
        },
        emit: (address, length) => {
          call('/emit', text(address, length));
        },
        fail: (address, length) => {
          call('/fail', text(address, length));
        },
      },
    });
    memory = instance.exports.memory;
    do {
      call('/calibrate');
      spin(2000);
    } while (call('/calibrated') === 'again');
    const status = instance.exports.main(0, 0);
    call('/done', String(status));
  } catch (error) {
    call('/error', String(error && error.stack ? error.stack : error));
  }
};
