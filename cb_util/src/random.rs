pub use rand::{Rng, RngCore};
pub use uuid::Uuid;
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

// Native builds take entropy from the OS. WebAssembly builds have no OS, and
// the `stdweb` feature rand 0.5 used for this is gone (stdweb itself is dead),
// so the browser gets a ChaCha stream seeded from Math.random instead.
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub use rand::thread_rng;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn thread_rng() -> ::rand::chacha::ChaChaRng {
    use ::rand::SeedableRng;
    use std::cell::RefCell;

    thread_local! {
        static RNG: RefCell<Option<::rand::chacha::ChaChaRng>> = RefCell::new(None);
    }

    RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        if rng.is_none() {
            let mut seed = [0u8; 32];
            for byte in seed.iter_mut() {
                *byte = (::js_sys::Math::random() * 256.0) as u8;
            }
            *rng = Some(::rand::chacha::ChaChaRng::from_seed(seed));
        }
        // Draw a new seed from the stored stream (which advances it) and hand out
        // a generator seeded from it. Simply cloning the stored generator, as
        // this used to do, returned an identical stream every time - so every
        // `uuid()` in the process was the same and, for example, a planned road
        // grid collapsed into a single gesture.
        let seed: [u8; 32] = rng.as_mut().unwrap().gen();
        ::rand::chacha::ChaChaRng::from_seed(seed)
    })
}

// A hashing function with hopefully low correlation between seeds
// but not necessarily good randomness of sequential probes on the same seed
pub struct FnvRng {
    seed: u64,
}

impl RngCore for FnvRng {
    fn next_u64(&mut self) -> u64 {
        let current = self.seed;
        let mut hasher = FnvHasher::default();
        self.seed.hash(&mut hasher);
        self.seed = hasher.finish();
        current
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn fill_bytes(&mut self, _bytes: &mut [u8]) {
        unimplemented!()
    }

    fn try_fill_bytes(&mut self, _bytes: &mut [u8]) -> Result<(), ::rand::Error> {
        unimplemented!()
    }
}

pub fn seed<S: Hash>(seed: S) -> FnvRng {
    let mut hasher = FnvHasher::default();
    seed.hash(&mut hasher);
    FnvRng {
        seed: hasher.finish(),
    }
}

pub fn uuid() -> Uuid {
    Uuid::from_random_bytes(thread_rng().gen())
}
