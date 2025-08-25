use std::io::{Read, Seek, Write};

use hound::{SampleFormat, WavSpec, WavWriter};

use kfc::io::ReadExt;

pub fn deserialize_audio<R: Read, W: Write + Seek>(
    mut reader: R,
    writer: W,
    channels: u16,
    sample_rate: u32,
    frame_count: u32
) -> anyhow::Result<()> {
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int
    };

    let writer = std::io::BufWriter::new(writer);
    let mut wav_writer = WavWriter::new(writer, spec)?;

    let sample_count = frame_count * channels as u32;

    for _ in 0..sample_count {
        let sample = reader.read_i16()?;
        wav_writer.write_sample(sample)?;
    }

    wav_writer.finalize()?;

    Ok(())
}
