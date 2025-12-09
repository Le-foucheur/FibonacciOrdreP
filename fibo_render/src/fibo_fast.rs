use algo::bit_iterator::BitIterable;
use num::{bigint::Sign, BigInt};

pub struct FiboFastManager {
    scratch1: Vec<u32>,
    scratch2: Vec<u32>,
    output: Vec<u32>,
}

impl FiboFastManager {
    pub fn new() -> FiboFastManager {
        FiboFastManager {
            scratch1: Vec::new(),
            scratch2: Vec::new(),
            output: Vec::new(),
        }
    }

    pub fn generate(&mut self, p: usize, n: usize, end: BigInt) -> impl Iterator<Item = bool> + use<'_> {
        // Generate the sequence

        self.output.resize(n.div_ceil(32), 0);
        let params = algo::setup(p);
        self.scratch1.resize(params.ranges_size, 0);
        self.scratch2.resize(params.ranges_size, 0);


        algo::calculator(
            self.scratch1.as_mut_slice(),
            self.scratch2.as_mut_slice(),
            &mut self.output,
            params,
            end.iter_u32_digits().rev().flat_map(
                |limb| limb.iter_bits().rev()
            ),
            end.sign() == Sign::Minus,
        );

        self.output.iter().copied().flat_map(|limb| limb.iter_bits()).take(n)
    }
}
