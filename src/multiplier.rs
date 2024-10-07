use knyst::prelude::*;

/// Processes audio samples by multiplying the input samples with the multiplier values and writes the result to the output buffers.
///
/// # Parameters
///
/// * `&mut self` - Mutable reference to self.
/// * `block_size` - The size of the audio block to process.
///
/// *inputs*
///
/// 0. `in_left`: Left channel
/// 1. `in_right`: Right channel
/// 2. `multiplier` - Slice of multiplier values for the audio samples.
///
/// *outputs*
///
/// 0. "out_left": Left channel
/// 1. "out_right": Right channel
///
/// # Returns
///
/// `GenState` - The state of the generator after processing.
pub struct Multiplier;

#[impl_gen]
impl Multiplier {
    #[process]
    fn process(
        #[allow(unused)] &mut self,
        block_size: BlockSize,
        #[allow(unused)] mut in_left: &[Sample],
        #[allow(unused)] mut in_right: &[Sample],
        #[allow(unused)] mut multiplier: &[Sample],
        #[allow(unused)] mut out_left: &mut [Sample],
        #[allow(unused)] mut out_right: &mut [Sample],
    ) -> GenState {
        for i in 0..*block_size {
            out_left[i] = multiplier[i] * in_left[i];
            out_right[i] = multiplier[i] * in_right[i];
        }
        GenState::Continue
    }
}
