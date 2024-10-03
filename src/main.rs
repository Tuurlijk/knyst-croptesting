use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use knyst::{
    audio_backend::{CpalBackend, CpalBackendOptions},
    graph::*,
    prelude::*,
};
use std::time::Duration;
use std::sync::mpsc::channel;

#[derive(Parser)]
#[clap(version, about)]
pub struct Args {
    /// Sound file to play
    #[clap(long, default_value = "sessions/LRMonoPhase4.wav")]
    file: String,
    /// Playback volume. Will use `Mult` node if volume is not 1.0
    #[clap(long, default_value = "1.0")]
    volume: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let (error_sender, _error_receiver) = channel();
    let mut backend = CpalBackend::new(CpalBackendOptions::default())?;
    let _sphere = KnystSphere::start(
        &mut backend,
        SphereSettings {
            num_inputs: 0,
            num_outputs: 1,
            ..Default::default()
        },
        Box::new(move |error| {
            error_sender.send(format!("{error}")).unwrap();
        }),
    );

    let sound_buffer = Buffer::from_sound_file(&args.file)?;
    let buffer_channels = sound_buffer.num_channels();
    let buffer = knyst_commands().insert_buffer(sound_buffer);

    let mut k = knyst_commands();

    let mut settings = k.default_graph_settings();
    settings.sample_rate = backend.sample_rate() as f32;
    settings.block_size = backend.block_size().unwrap_or(64);
    settings.num_outputs = buffer_channels;
    settings.num_inputs = 0;
    let mut graph = Graph::new(settings);

    let buf_playback_node = BufferReaderMulti::new(buffer, 1.0, StopAction::FreeSelf)
        .channels(buffer_channels)
        .looping(false);
    let playback_node_id = graph.push(buf_playback_node);

    if args.volume == 1.0 {
        println!("Outputting raw file");
        for i in 0..buffer_channels {
            graph.connect(playback_node_id.to_graph_out().from_index(i).to_index(i))?;
        }
    } else {
        println!("Outputting through `Mult` with multiplier of {:?}", args.volume);
        for i in 0..buffer_channels {
            let amp = graph.push(Mult);
            graph.connect(amp.to_graph_out().to_index(i))?;
            graph.connect(playback_node_id.to(amp).from_index(i).to_index(0))?;
            graph.connect(constant(args.volume).to(amp).to_index(1))?;
        }
    }

    let note_graph_id = k.push(graph, inputs!());
    k.connect(note_graph_id.to_graph_out().channels(buffer_channels));

    let total_duration = buffer.duration().to_seconds_f64() as u64;
    let pb = ProgressBar::new(total_duration);
    pb.set_style(ProgressStyle::with_template("{elapsed_precise} / {duration_precise} [{wide_bar}]")?.progress_chars("█▉▊▋▌▍▎▏ "));
    for _ in 0..total_duration {
        std::thread::sleep(Duration::from_secs(1));
        pb.inc(1);
    }
    std::thread::sleep(Duration::from_secs(1));
    pb.finish();
    Ok(())
}