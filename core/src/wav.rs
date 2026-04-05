use std::io::Cursor;

/// PCM f32 サンプル列を WAV バイト列にエンコードする
///
/// - 出力フォーマット: モノラル, 指定サンプルレート, 指定ビット深度
/// - hound crate を使用
pub fn encode_wav(samples: &[f32], sample_rate: u32, bit_depth: u16) -> Result<Vec<u8>, String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: bit_depth,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buf = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut buf, spec)
            .map_err(|e| format!("WAV writer init failed: {}", e))?;

        let max_val = (1i32 << (bit_depth - 1)) - 1;
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * max_val as f32) as i32;
            writer
                .write_sample(int_sample)
                .map_err(|e| format!("WAV write failed: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("WAV finalize failed: {}", e))?;
    }

    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_produces_valid_wav() {
        let samples: Vec<f32> = (0..44100)
            .map(|i| (i as f32 / 44100.0 * 440.0 * std::f32::consts::TAU).sin() * 0.5)
            .collect();

        let wav = encode_wav(&samples, 44100, 16).unwrap();
        // WAV ファイルは RIFF ヘッダで始まる
        assert_eq!(&wav[0..4], b"RIFF");
        // 最低でもヘッダ + データがある
        assert!(wav.len() > 44);
    }

    #[test]
    fn encode_empty_samples() {
        let wav = encode_wav(&[], 44100, 16).unwrap();
        assert_eq!(&wav[0..4], b"RIFF");
    }
}
