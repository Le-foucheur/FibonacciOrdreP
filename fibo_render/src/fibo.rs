pub struct FiboManager {
    pub p: u64,
    pub sequences: Vec<FiboSequence>,
}

impl FiboManager {
    pub fn new() -> FiboManager {
        FiboManager {
            p: 1,
            sequences: vec![FiboSequence::new(1)],
        }
    }

    pub fn generate(&mut self, p: u64, n: u64, start: u64) -> Vec<bool> {
        if self.p < p {
            // Extend the sequences
            for i in self.p..p {
                self.sequences.push(FiboSequence::new(i + 1));
            }
            self.p = p;
        }
        // Generate the sequence
        self.sequences[p as usize - 1].generate(n, start)
    }
}

#[derive(Clone)]
pub struct FiboSequence {
    pub p: u64,
    pub n: u64,
    pub sequence: Vec<bool>,
}

impl FiboSequence {
    pub fn new(p: u64) -> FiboSequence {
        let size = if p == 0 { 1 } else { p };
        // let mut seq = vec![false; (size-1) as usize];
        // seq.push(true);
        let seq: Vec<bool> = vec![true; size as usize];
        FiboSequence {
            p,
            n: size,
            sequence: seq,
        }
    }

    pub fn generate(&mut self, n: u64, start: u64) -> Vec<bool> {
        // Extend the sequence
        if self.n < n {
            self.sequence.resize(n as usize, true);
            for i in self.n as usize..n as usize {
                self.sequence[i] = self.sequence[i - 1] ^ self.sequence[i - self.p as usize];
            }
            self.n = n;
        }
        // Return the sequence after start index
        self.sequence[start as usize..].to_vec()
    }
}


pub fn generate_fibo(p: u64, n: u64) -> Vec<u128> {
    let mut result = vec![1; n as usize];
    for i in p as usize..n as usize {
        result[i] = result[i - 1] + result[i - p as usize];
    }
    return result;
}