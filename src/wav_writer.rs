use hound;
use knyst::prelude::*;

/// Takes input samples and writes them to the WAV file. Passes through input to output.
///
/// # Parameters
///
/// *inputs*
///
/// 0. `in_left`: Left channel
/// 1. `in_right`: Right channel
/// 2. `block_size`: The number of samples to process.
///
/// *outputs*
///
/// 0. `out_left`: Left channel
/// 1. `out_right`: Right channel
///
/// # Returns
///
/// - `GenState`: The state indicating whether to continue or stop the generator.
pub struct WavWriterGen {
    writer: Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>,
    filename: String,
}

#[impl_gen]
impl WavWriterGen {
    /// Creates a new `WavWriterGen` with the specified filename.
    #[new]
    pub fn new(filename: String) -> Self {
        Self {
            writer: None,
            filename,
        }
    }

    /// Initializes the WAV writer with the correct sample rate.
    ///
    /// # Parameters
    ///
    /// - `sample_rate`: The sample rate in samples per second.
    /// - `node_id`: The ID of the node associated with this generator.
    #[init]
    fn init(&mut self, sample_rate: SampleRate, _node_id: NodeId) {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: *sample_rate as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        self.writer = Some(hound::WavWriter::create(&self.filename, spec).unwrap());
    }

    #[process]
    fn process(
        &mut self,
        in_left: &[Sample],
        in_right: &[Sample],
        out_left: &mut [Sample],
        out_right: &mut [Sample],
        block_size: BlockSize,
    ) -> GenState {
        if let Some(ref mut writer) = self.writer {
            for i in 0..*block_size {
                let left_sample = in_left[i];
                let right_sample = in_right[i];

                // Write to WAV file
                let left_sample_i16 = (left_sample.clamp(-1.0, 1.0) * i16::MAX as Sample) as i16;
                let right_sample_i16 = (right_sample.clamp(-1.0, 1.0) * i16::MAX as Sample) as i16;

                if let Err(e) = writer.write_sample(left_sample_i16) {
                    eprintln!("Error writing left sample: {}", e);
                }
                if let Err(e) = writer.write_sample(right_sample_i16) {
                    eprintln!("Error writing right sample: {}", e);
                }

                // Pass through the input signal
                out_left[i] = left_sample;
                out_right[i] = right_sample;
            }
            writer.flush().unwrap();
        }
        GenState::Continue
    }
}

impl Drop for WavWriterGen {
    fn drop(&mut self) {
        if let Some(writer) = self.writer.take() {
            println!("Finalizing wav writer");
            if let Err(e) = writer.finalize() {
                eprintln!("Error finalizing WAV writer: {}", e);
            }
        }
    }
}
