use std::time::SystemTime;

pub(crate) union MWC64X {
    vector: (u32, u32),
    scalar: u64,
}

impl MWC64X {
    pub(crate) fn new(seed: Option<u64>) -> MWC64X {
        let seed = match seed {
            Some(s) => s,
            None => SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Duration since UNIX_EPOCH failed")
                .as_secs(),
        };
        
        MWC64X { scalar: seed ^ 0x8CCC1D021231BBAC}
    }

    pub(crate) fn state(&self) -> u64 {
        unsafe { self.scalar }
    }

    pub(crate) fn skip(&mut self, distance: u64) {
        // a < m && b < m -> r = (a+b) % m
        #[inline]
        fn modular_add64(a: u64, b: u64, m: u64) -> u64 {
            let mut res = a + b;

            if res >= m || res < a {
                res -= m;
            }

            res
        }
        
        // a < m && b < m -> (a*b) % m
        #[inline]
        fn modular_mul64(mut a: u64, mut b: u64, m: u64) -> u64 {
            let mut res = 0;

            while a != 0 {
                if a & 1 != 0 {
                    res = modular_add64(res, b, m);
                }
                b = modular_add64(b, b, m);
                a >>= 1;
            }

            res
        }
        
        // a < m && e < m -> (a**e) % m
        #[inline]
        fn modular_pow64(a: u64, mut e: u64, m: u64) -> u64 {
            let (mut sqr, mut acc) = (a, 1);

            while e != 0 {
                if e & 1 != 0 {
                    acc = modular_mul64(acc, sqr, m);
                }
                sqr = modular_mul64(sqr, sqr, m);
                e >>= 1;
            }

            acc
        }

        const A: u64 = 0xFFFEB81B;
        const M: u64 = 0xFFFEB81AFFFFFFFF;

        let m = modular_pow64(A, distance, M);
        let state = unsafe { self.vector };
        let mut x = state.0 as u64 * A + state.1 as u64;
        x = modular_mul64(x, m, M);
        self.vector = ((x / A) as u32, (x % A) as u32);
    }
}