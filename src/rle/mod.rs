#[derive(Debug)]
pub(crate) struct RleRun {
    pub(crate) value: u8,
    pub(crate) count: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RleSequence(Vec<RleRun>);

impl RleSequence {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn empty() -> Self {
        RleSequence(Vec::new())
    }
}

impl From<&[u8]> for RleSequence {
    fn from(data: &[u8]) -> Self {
        let mut sequence = Vec::new();
        if !data.is_empty() {
            let mut current_value = data[0];
            let mut current_count = 1;

            for &byte in &data[1..] {
                if byte == current_value {
                    current_count += 1;
                } else {
                    sequence.push(RleRun {
                        value: current_value,
                        count: current_count,
                    });
                    current_value = byte;
                    current_count = 1;
                }
            }

            sequence.push(RleRun {
                value: current_value,
                count: current_count,
            });
        }

        RleSequence(sequence)
    }
}

impl Into<Vec<u8>> for RleSequence {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();
        for run in self.0 {
            data.extend(std::iter::repeat(run.value).take(run.count as usize));
        }
        data
    }
}
