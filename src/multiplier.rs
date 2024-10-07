use knyst::prelude::impl_gen;
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

#[cfg(test)]
mod tests {
    use crate::multiplier::Multiplier;
    use knyst::{knyst_commands, offline::KnystOffline, prelude::*};

    #[test]
    fn multiplier_test() {
        let sample_rate = 44100;
        let buffer_size = 2;
        let buffer_channels = 2;

        let mut ko = KnystOffline::new(
            sample_rate,
            buffer_size,
            0,
            buffer_channels,
        );

        let mut k = knyst_commands();
        let mut settings = k.default_graph_settings();
        settings.sample_rate = sample_rate as Sample;
        settings.block_size = buffer_size;
        settings.num_outputs = 2;
        settings.num_inputs = 0;
        let mut graph = Graph::new(settings);

        let amp = graph.push(Multiplier);
        graph.connect(amp.to_graph_out().from_index(0).to_index(0)).unwrap();
        graph.connect(constant(2.0).to(amp).to_index(0)).unwrap();
        graph.connect(constant(3.0).to(amp).to_label("multiplier")).unwrap();

        graph.connect(constant(21.0).to(amp).to_index(1)).unwrap();
        graph.connect(amp.to_graph_out().from_index(1).to_index(1)).unwrap();

        let graph_id = k.push(graph, inputs!());
        k.connect(graph_id.to_graph_out().channels(buffer_channels));

        ko.process_block();

        let o = ko.output_channel(0).unwrap();
        dbg!(o);
        assert_eq!(o[0], 6.0);

        let o1 = ko.output_channel(1).unwrap();
        dbg!(o1);
        assert_eq!(o1[0], 63.0);
    }
}
