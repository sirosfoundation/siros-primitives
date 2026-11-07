//! FNV-1a over 128 bits: a stable hash, which a new compiler leaves as it was, of what a module
//! holds and of what a record keeps.

/// The FNV-1a state.
#[derive(Clone, Copy, Debug)]
pub struct Fingerprint(u128);

/// The offset basis of 128-bit FNV.
const BASIS: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;

/// The prime of 128-bit FNV.
const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

impl Fingerprint {
    /// The state before any byte.
    #[must_use]
    pub const fn new() -> Self {
        Self(BASIS)
    }

    /// Takes in `bytes`.
    pub fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0 ^ u128::from(*byte)).wrapping_mul(PRIME);
        }
    }

    /// Takes in another hash.
    pub fn update_hash(&mut self, hash: u128) {
        self.update(&hash.to_le_bytes());
    }

    /// The hash of the bytes so far.
    #[must_use]
    pub const fn finish(self) -> u128 {
        self.0
    }

    /// The hash of `bytes`.
    #[must_use]
    pub fn of(bytes: &[u8]) -> u128 {
        let mut state = Self::new();
        state.update(bytes);
        state.finish()
    }
}

impl Default for Fingerprint {
    fn default() -> Self {
        Self::new()
    }
}
