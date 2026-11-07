// The memory that the measuring worker shares with the helper worker: the state at byte 0, and
// the cycles, instructions and branch misses of the last `/end` from byte 8.

'use strict';

const IDLE = 0;
const BEGIN = 1;
const END = 2;
const DONE = 3;
const FAILED = 4;

const views = (buffer) => ({
  state: new Int32Array(buffer, 0, 1),
  counts: new BigUint64Array(buffer, 8, 3),
});
