use crate::params::WaveType;

/// 波形サンプルを生成する（Bfxr AS3 ソースに準拠）
///
/// - `phase`: 現在の位相（0 から period まで）
/// - `period`: 波形の周期（supersampling レートでのサンプル数）
/// - `duty_cycle`: 矩形波のデューティ比（内部値: 0.0〜1.0）
/// - `noise_buffer`: ノイズ波形用の 32 サンプルバッファ
pub fn oscillate(
    wave_type: WaveType,
    phase: f64,
    period: f64,
    duty_cycle: f64,
    noise_buffer: &[f64; 32],
) -> f64 {
    match wave_type {
        WaveType::Square => square(phase, period, duty_cycle),
        WaveType::Sine => sine(phase, period),
        WaveType::Noise => noise(phase, period, noise_buffer),
        WaveType::Sawtooth => sawtooth(phase, period),
        WaveType::Triangle => triangle(phase, period),
    }
}

/// 矩形波: Bfxr Case 0
/// `(phase / period) < duty_cycle ? 0.5 : -0.5`
fn square(phase: f64, period: f64, duty_cycle: f64) -> f64 {
    if (phase / period) < duty_cycle {
        0.5
    } else {
        -0.5
    }
}

/// 正弦波: Bfxr Case 2（高速近似アルゴリズム）
/// 標準の sin() ではなく、Bfxr 互換の多項式近似を使用
fn sine(phase: f64, period: f64) -> f64 {
    let mut pos = phase / period;
    pos = if pos > 0.5 {
        (pos - 1.0) * std::f64::consts::TAU
    } else {
        pos * std::f64::consts::TAU
    };

    // First approximation: Bhaskara-like
    let temp = if pos < 0.0 {
        1.27323954 * pos + 0.405284735 * pos * pos
    } else {
        1.27323954 * pos - 0.405284735 * pos * pos
    };

    // Second pass: refine accuracy
    if temp < 0.0 {
        0.225 * (temp * -temp - temp) + temp
    } else {
        0.225 * (temp * temp - temp) + temp
    }
}

/// ホワイトノイズ: Bfxr Case 3
/// 32 サンプルバッファからルックアップ
fn noise(phase: f64, period: f64, buffer: &[f64; 32]) -> f64 {
    let index = (phase * 32.0 / period) as usize % 32;
    buffer[index]
}

/// 鋸歯状波: Bfxr Case 1
/// `1.0 - (phase / period) * 2.0`
fn sawtooth(phase: f64, period: f64) -> f64 {
    1.0 - (phase / period) * 2.0
}

/// 三角波: Bfxr Case 4
/// `abs(1 - (phase / period) * 2) - 1`
fn triangle(phase: f64, period: f64) -> f64 {
    (1.0 - (phase / period) * 2.0).abs() - 1.0
}

/// ノイズバッファを乱数で初期化する
pub fn fill_noise_buffer(buffer: &mut [f64; 32], rng: &mut impl FnMut() -> f64) {
    for sample in buffer.iter_mut() {
        *sample = rng() * 2.0 - 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_wave_values() {
        let noise = [0.0f64; 32];
        // duty_cycle = 0.5 → 前半 0.5, 後半 -0.5
        assert_eq!(oscillate(WaveType::Square, 0.0, 100.0, 0.5, &noise), 0.5);
        assert_eq!(oscillate(WaveType::Square, 25.0, 100.0, 0.5, &noise), 0.5);
        assert_eq!(oscillate(WaveType::Square, 49.0, 100.0, 0.5, &noise), 0.5);
        assert_eq!(oscillate(WaveType::Square, 50.0, 100.0, 0.5, &noise), -0.5);
        assert_eq!(oscillate(WaveType::Square, 75.0, 100.0, 0.5, &noise), -0.5);
    }

    #[test]
    fn sine_wave_range() {
        let noise = [0.0f64; 32];
        let period = 100.0;
        for i in 0..100 {
            let val = oscillate(WaveType::Sine, i as f64, period, 0.5, &noise);
            assert!(val >= -1.0 && val <= 1.0, "sine out of range at phase {}: {}", i, val);
        }
    }

    #[test]
    fn sine_wave_zero_crossing() {
        let noise = [0.0f64; 32];
        // phase=0 should be near 0
        let val = oscillate(WaveType::Sine, 0.0, 100.0, 0.5, &noise);
        assert!(val.abs() < 0.01, "sine at phase 0 should be near 0: {}", val);
    }

    #[test]
    fn noise_wave_uses_buffer() {
        let mut noise = [0.0f64; 32];
        noise[0] = 0.42;
        let val = oscillate(WaveType::Noise, 0.0, 100.0, 0.5, &noise);
        assert!((val - 0.42).abs() < 1e-10);
    }

    #[test]
    fn sawtooth_wave_range() {
        let noise = [0.0f64; 32];
        let period = 100.0;
        for i in 0..100 {
            let val = oscillate(WaveType::Sawtooth, i as f64, period, 0.5, &noise);
            assert!(val >= -1.0 && val <= 1.0, "sawtooth out of range at {}: {}", i, val);
        }
    }

    #[test]
    fn triangle_wave_range() {
        let noise = [0.0f64; 32];
        let period = 100.0;
        for i in 0..100 {
            let val = oscillate(WaveType::Triangle, i as f64, period, 0.5, &noise);
            assert!(val >= -1.0 && val <= 1.0, "triangle out of range at {}: {}", i, val);
        }
    }
}
