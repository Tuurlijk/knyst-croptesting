mod wav_writer;
mod multiplier;

use crate::multiplier::Multiplier;
use crate::wav_writer::WavWriterGen;
use anyhow::Result;
use clap::Parser;
use knyst::offline::KnystOffline;
use knyst::{
    graph::*,
    prelude::*,
};

#[derive(Parser)]
#[clap(version, about)]
pub struct Args {
    /// Sound file to play
    #[clap(long, default_value = "sessions/LRMonoPhase4.wav")]
    file: String,
    /// Playback volume. Will use the specified multiplier node if volume is not 1.0
    #[clap(long, default_value = "1.0")]
    volume: f32,
    /// RenderType; either `file` or `tone`
    #[clap(long, default_value = "tone")]
    render_type: String,
    /// Sample Rate
    #[clap(long, default_value = "48000")]
    sample_rate: usize,
    /// Buffer Size
    #[clap(long, default_value = "48000")]
    buffer_size: usize,
    /// Multiplier
    #[clap(long, default_value = "mult")]
    multiplier: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let sample_rate = args.sample_rate;
    let buffer_size = args.buffer_size;

    let mut ko = KnystOffline::new(
        sample_rate,
        buffer_size,
        0,
        1,
    );

    let sound_buffer = Buffer::from_sound_file(&args.file)?;
    let buffer_channels = sound_buffer.num_channels();
    let buffer = knyst_commands().insert_buffer(sound_buffer);

    let mut k = knyst_commands();
    let mut settings = k.default_graph_settings();
    settings.sample_rate = sample_rate as Sample;
    settings.block_size = buffer_size;
    settings.num_outputs = buffer_channels;
    settings.num_inputs = 0;
    let mut graph = Graph::new(settings);

    match args.render_type.as_str() {
        "file" => {
            let buf_playback_node = BufferReaderMulti::new(buffer, 1.0, StopAction::FreeSelf)
                .channels(buffer_channels)
                .looping(false);
            let playback_node = graph.push(buf_playback_node);


            if args.volume == 1.0 {
                let wav_writer = WavWriterGen::new("croptesting_out_file_plain.wav".to_string());
                let wav_writer_node = graph.push(wav_writer);

                println!("Outputting unaltered samples");
                for i in 0..buffer_channels {
                    graph.connect(wav_writer_node.to_graph_out().from_index(i).to_index(i))?;
                    graph.connect(playback_node.to(wav_writer_node).from_index(i).to_index(i))?;
                }
            } else {
                let wav_writer = WavWriterGen::new(format!("croptesting_out_file_{}.wav", args.multiplier));
                let wav_writer_node = graph.push(wav_writer);

                match args.multiplier.as_str() {
                    "mult" => {
                        println!("Outputting through `Mult` with multiplier of {:?}", args.volume);
                        for i in 0..buffer_channels {
                            let amp = graph.push(Mult);
                            graph.connect(wav_writer_node.to_graph_out().from_index(i).to_index(i))?;
                            graph.connect(amp.to(wav_writer_node).to_index(i))?;
                            graph.connect(playback_node.to(amp).from_index(i).to_index(0))?;
                            graph.connect(constant(args.volume).to(amp).to_index(1))?;
                        }
                    }
                    "multiplier" | _ => {
                        println!("Outputting through `Multiplier` with multiplier of {:?}", args.volume);
                        let amp = graph.push(Multiplier);
                        for i in 0..buffer_channels {
                            graph.connect(wav_writer_node.to_graph_out().from_index(i).to_index(i))?;
                            graph.connect(amp.to(wav_writer_node).from_index(i).to_index(i))?;
                            graph.connect(playback_node.to(amp).from_index(i).to_index(i))?;
                            graph.connect(constant(args.volume).to(amp).to_label("multiplier"))?;
                        }
                    }
                }
            }
        }
        "tone" | _ => {
            let osc = Oscillator::new(WavetableId::cos());
            let osc_node = graph.push(osc);

            if args.volume == 1.0 {
                let wav_writer = WavWriterGen::new("croptesting_out_tone_plain.wav".to_string());
                let wav_writer_node = graph.push(wav_writer);

                for i in 0..buffer_channels {
                    graph.connect(wav_writer_node.to_graph_out().from_index(i).to_index(i))?;
                    graph.connect(constant(110 as Sample).to(osc_node).to_label("freq"))?;
                    graph.connect(osc_node.to(wav_writer_node).to_index(i))?;
                }
            } else {
                let wav_writer = WavWriterGen::new(format!("croptesting_out_tone_{}.wav", args.multiplier));
                let wav_writer_node = graph.push(wav_writer);

                match args.multiplier.as_str() {
                    "mult" => {
                        println!("Outputting tone using two `Mult` amps inside the loop.");
                        for i in 0..buffer_channels {
                            let amp = graph.push(Mult);
                            graph.connect(wav_writer_node.to_graph_out().from_index(i).to_index(i))?;
                            graph.connect(amp.to(wav_writer_node).to_index(i))?;
                            graph.connect(constant(110 as Sample).to(osc_node).to_label("freq"))?;
                            graph.connect(osc_node.to(amp).to_index(0))?;
                            graph.connect(constant(args.volume).to(amp).to_index(1))?;
                        }
                    }
                    "multiplier" | _ => {
                        println!("Outputting tone using a single `Multiplier` amp outside the loop.");
                        let amp = graph.push(Multiplier);
                        for i in 0..buffer_channels {
                            graph.connect(wav_writer_node.to_graph_out().from_index(i).to_index(i))?;
                            graph.connect(amp.to(wav_writer_node).from_index(i).to_index(i))?;
                            graph.connect(constant(110 as Sample).to(osc_node).to_label("freq"))?;
                            graph.connect(osc_node.to(amp).to_index(i))?;
                            graph.connect(constant(args.volume).to(amp).to_label("multiplier"))?;
                        }
                    }
                }
            }
        }
    }

    let note_graph_id = k.push(graph, inputs!());
    k.connect(note_graph_id.to_graph_out().channels(buffer_channels));

    let total_duration = ((buffer.duration().to_seconds_f64() * sample_rate as f64) / buffer_size as f64) as u64;

    for _ in 0..total_duration {
        ko.process_block();
    }
    ko.process_block();

    Ok(())
}