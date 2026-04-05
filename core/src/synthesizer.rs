use crate::envelope::Envelope;
use crate::oscillator::{self, fill_noise_buffer};
use crate::params::SfxParams;

/// 合成エンジンの内部状態（Bfxr の SfxrSynth 相当）
struct SynthState {
    // 周波数（period ベース）
    period: f64,
    max_period: f64,
    slide: f64,
    delta_slide: f64,

    // ビブラート
    vibrato_phase: f64,
    vibrato_speed: f64,
    vibrato_amplitude: f64,

    // 波形
    phase: f64,
    duty_cycle: f64,
    duty_sweep: f64,

    // ノイズバッファ
    noise_buffer: [f64; 32],

    // ミュート
    muted: bool,
    min_frequency: f64,
}

/// パラメータから PCM サンプル列を生成する
///
/// Bfxr の synthWave() に対応。8x supersampling で内部処理し、
/// 出力サンプルレートは 44100 Hz。
pub fn generate(params: &SfxParams) -> Vec<f32> {
    let mut envelope = Envelope::new(&params.envelope);
    let total_samples = envelope.total_samples();

    if total_samples == 0 {
        return Vec::new();
    }

    // --- パラメータ変換（bfxr-mapping.md Section 2 準拠） ---

    // 周波数: _period = 100.0 / (f^2 + 0.001)
    let period = 100.0 / (params.frequency.base * params.frequency.base + 0.001);
    let max_period = if params.frequency.limit > 0.0 {
        100.0 / (params.frequency.limit * params.frequency.limit + 0.001)
    } else {
        f64::MAX
    };

    // スライド: _slide = 1.0 - param^3 * 0.01
    let slide_param = params.frequency.slide;
    let slide = 1.0 - slide_param * slide_param * slide_param * 0.01;

    // デルタスライド: _deltaSlide = -param^3 * 0.000001
    let ds_param = params.frequency.delta_slide;
    let delta_slide = -ds_param * ds_param * ds_param * 0.000001;

    // ビブラート
    let vibrato_amplitude = params.frequency.vibrato_depth * 0.5;
    let vibrato_speed = params.frequency.vibrato_speed * params.frequency.vibrato_speed * 0.01;

    // デューティサイクル内部値: 0.5 - param * 0.5
    let duty_cycle = 0.5 - params.wave.duty_cycle * 0.5;
    // デューティスイープ: -param * 0.00005
    let duty_sweep = -params.wave.duty_sweep * 0.00005;

    // ノイズバッファ初期化
    let mut noise_buffer = [0.0f64; 32];
    let mut rng_state: u64 = 42; // 固定シード（再現性のため）
    fill_noise_buffer(&mut noise_buffer, &mut || {
        // 簡易 xorshift64 乱数
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 7;
        rng_state ^= rng_state << 17;
        (rng_state as f64) / (u64::MAX as f64)
    });

    let mut state = SynthState {
        period,
        max_period,
        slide,
        delta_slide,
        vibrato_phase: 0.0,
        vibrato_speed,
        vibrato_amplitude,
        phase: 0.0,
        duty_cycle,
        duty_sweep,
        noise_buffer,
        muted: false,
        min_frequency: params.frequency.limit,
    };

    let volume = params.output.volume;
    let wave_type = params.wave.wave_type;

    let mut output = Vec::with_capacity(total_samples);

    // --- メインループ ---
    for _ in 0..total_samples {
        if envelope.is_finished() {
            break;
        }

        // 6. 周波数スライド
        state.slide += state.delta_slide;
        state.period *= state.slide;

        // 7. 周波数下限チェック
        if state.period > state.max_period {
            state.period = state.max_period;
            if state.min_frequency > 0.0 {
                state.muted = true;
            }
        }

        // 8. ビブラート
        let period_temp = if state.vibrato_amplitude > 0.0 {
            state.vibrato_phase += state.vibrato_speed;
            state.period * (1.0 + state.vibrato_phase.sin() * state.vibrato_amplitude)
        } else {
            state.period
        };

        // 9. 周期の整数化と最小値制限
        let period_temp = (period_temp as i64).max(8) as f64;

        // 10. デューティスイープ
        state.duty_cycle += state.duty_sweep;
        state.duty_cycle = state.duty_cycle.clamp(0.0, 0.5);

        // 11. エンベロープ更新
        let envelope_vol = envelope.tick();

        // 14-15. 8x supersampling + 波形生成
        let mut super_sample = 0.0f64;
        for _ in 0..8 {
            // 位相進行
            state.phase += 1.0;

            // 周期越えで位相リセット + ノイズ再生成
            if state.phase >= period_temp {
                state.phase -= period_temp;

                // ノイズバッファ再生成
                if wave_type == crate::params::WaveType::Noise {
                    fill_noise_buffer(&mut state.noise_buffer, &mut || {
                        rng_state ^= rng_state << 13;
                        rng_state ^= rng_state >> 7;
                        rng_state ^= rng_state << 17;
                        (rng_state as f64) / (u64::MAX as f64)
                    });
                }
            }

            // 波形生成（オーバートーンなし、k=0 のみ）
            let sample = oscillator::oscillate(
                wave_type,
                state.phase,
                period_temp,
                state.duty_cycle,
                &state.noise_buffer,
            );

            super_sample += sample;
        }

        // 19. クリッピング
        super_sample = super_sample.clamp(-8.0, 8.0);

        // 20. 音量適用: volume * envelope_vol * super_sample * 0.125
        let final_sample = if state.muted {
            0.0
        } else {
            volume * envelope_vol * super_sample * 0.125
        };

        // クランプして出力
        output.push(final_sample.clamp(-1.0, 1.0) as f32);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::parse_params;

    #[test]
    fn generate_produces_samples() {
        let json = include_bytes!("../../examples/coin.json");
        let params = parse_params(json).unwrap();
        let samples = generate(&params);
        assert!(!samples.is_empty(), "should produce samples");
        assert!(samples.len() > 100, "coin sound should be more than 100 samples");
    }

    #[test]
    fn samples_in_range() {
        let json = include_bytes!("../../examples/coin.json");
        let params = parse_params(json).unwrap();
        let samples = generate(&params);
        for (i, &s) in samples.iter().enumerate() {
            assert!(s >= -1.0 && s <= 1.0, "sample {} out of range: {}", i, s);
        }
    }

    #[test]
    fn not_silence() {
        let json = include_bytes!("../../examples/coin.json");
        let params = parse_params(json).unwrap();
        let samples = generate(&params);
        let max_abs = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_abs > 0.01, "output should not be silence, max_abs={}", max_abs);
    }

    #[test]
    fn sine_wave_generates() {
        let json = br#"{"wave":{"type":"sine"},"envelope":{"sustain":0.1,"decay":0.1},"frequency":{"base":0.5}}"#;
        let params = parse_params(json).unwrap();
        let samples = generate(&params);
        assert!(!samples.is_empty());
        let max_abs = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(max_abs > 0.01);
    }
}
