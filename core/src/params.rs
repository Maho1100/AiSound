use serde::Deserialize;

/// 波形タイプ（Bfxr 互換の整数値に対応）
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WaveType {
    #[default]
    Square,
    Sawtooth,
    Sine,
    Noise,
    Triangle,
    // Phase 2+ で追加: PinkNoise, Tan, Whistle, Breaker, Bitnoise, Randomized, Buzz
}

/// トップレベルのパラメータファイル構造
#[derive(Debug, Clone, Deserialize)]
pub struct SfxParams {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
    #[serde(default)]
    pub wave: WaveParams,
    #[serde(default)]
    pub envelope: EnvelopeParams,
    #[serde(default)]
    pub frequency: FrequencyParams,
    #[serde(default)]
    pub arpeggio: ArpeggioParams,
    #[serde(default)]
    pub filter: FilterParams,
    #[serde(default)]
    pub phaser: PhaserParams,
    #[serde(default)]
    pub retrigger: RetriggerParams,
    #[serde(default)]
    pub distortion: DistortionParams,
    #[serde(default)]
    pub bitcrusher: BitcrusherParams,
    #[serde(default)]
    pub output: OutputParams,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WaveParams {
    #[serde(rename = "type", default)]
    pub wave_type: WaveType,
    #[serde(default = "default_duty_cycle")]
    pub duty_cycle: f64,
    #[serde(default)]
    pub duty_sweep: f64,
}

impl Default for WaveParams {
    fn default() -> Self {
        Self {
            wave_type: WaveType::Square,
            duty_cycle: 0.5,
            duty_sweep: 0.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvelopeParams {
    #[serde(default)]
    pub attack: f64,
    #[serde(default = "default_sustain")]
    pub sustain: f64,
    #[serde(default)]
    pub sustain_punch: f64,
    #[serde(default = "default_decay")]
    pub decay: f64,
}

impl Default for EnvelopeParams {
    fn default() -> Self {
        Self {
            attack: 0.0,
            sustain: 0.3,
            sustain_punch: 0.0,
            decay: 0.4,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FrequencyParams {
    #[serde(default = "default_freq_base")]
    pub base: f64,
    #[serde(default)]
    pub limit: f64,
    #[serde(default)]
    pub slide: f64,
    #[serde(default)]
    pub delta_slide: f64,
    #[serde(default)]
    pub vibrato_depth: f64,
    #[serde(default)]
    pub vibrato_speed: f64,
    #[serde(default)]
    pub vibrato_delay: f64,
}

impl Default for FrequencyParams {
    fn default() -> Self {
        Self {
            base: 0.3,
            limit: 0.0,
            slide: 0.0,
            delta_slide: 0.0,
            vibrato_depth: 0.0,
            vibrato_speed: 0.0,
            vibrato_delay: 0.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ArpeggioParams {
    #[serde(default)]
    pub multiplier: f64,
    #[serde(default)]
    pub speed: f64,
    #[serde(default)]
    pub limit: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterParams {
    #[serde(default = "default_one")]
    pub cutoff: f64,
    #[serde(default)]
    pub cutoff_sweep: f64,
    #[serde(default)]
    pub resonance: f64,
    #[serde(default)]
    pub highpass_cutoff: f64,
    #[serde(default)]
    pub highpass_sweep: f64,
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            cutoff: 1.0,
            cutoff_sweep: 0.0,
            resonance: 0.0,
            highpass_cutoff: 0.0,
            highpass_sweep: 0.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PhaserParams {
    #[serde(default)]
    pub offset: f64,
    #[serde(default)]
    pub sweep: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RetriggerParams {
    #[serde(default)]
    pub repeat_speed: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistortionParams {
    #[serde(default)]
    pub gain: f64,
    #[serde(default = "default_compress")]
    pub compress_ratio: f64,
}

impl Default for DistortionParams {
    fn default() -> Self {
        Self {
            gain: 0.0,
            compress_ratio: 0.5,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BitcrusherParams {
    #[serde(default = "default_bit_depth")]
    pub bit_depth: u32,
    #[serde(default)]
    pub sample_rate_reduction: f64,
}

impl Default for BitcrusherParams {
    fn default() -> Self {
        Self {
            bit_depth: 16,
            sample_rate_reduction: 0.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputParams {
    #[serde(default = "default_one")]
    pub volume: f64,
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    #[serde(default = "default_bit_depth")]
    pub bit_depth: u32,
}

impl Default for OutputParams {
    fn default() -> Self {
        Self {
            volume: 1.0,
            sample_rate: 44100,
            bit_depth: 16,
        }
    }
}

// Default value helper functions for serde
fn default_duty_cycle() -> f64 { 0.5 }
fn default_sustain() -> f64 { 0.3 }
fn default_decay() -> f64 { 0.4 }
fn default_freq_base() -> f64 { 0.3 }
fn default_one() -> f64 { 1.0 }
fn default_compress() -> f64 { 0.5 }
fn default_sample_rate() -> u32 { 44100 }
fn default_bit_depth() -> u32 { 16 }

/// JSON バイト列からパラメータを読み込む
pub fn parse_params(json: &[u8]) -> Result<SfxParams, String> {
    serde_json::from_slice(json).map_err(|e| format!("JSON parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_coin_json() {
        let json = include_bytes!("../../examples/coin.json");
        let params = parse_params(json).unwrap();
        assert_eq!(params.wave.wave_type, WaveType::Square);
        assert!((params.wave.duty_cycle - 0.5).abs() < 1e-10);
        assert!((params.envelope.sustain - 0.1).abs() < 1e-10);
        assert!((params.frequency.base - 0.3).abs() < 1e-10);
        assert!((params.frequency.slide - 0.35).abs() < 1e-10);
    }

    #[test]
    fn default_values() {
        let json = br#"{"wave":{"type":"sine"}}"#;
        let params = parse_params(json).unwrap();
        assert_eq!(params.wave.wave_type, WaveType::Sine);
        assert!((params.envelope.sustain - 0.3).abs() < 1e-10);
        assert!((params.envelope.decay - 0.4).abs() < 1e-10);
        assert_eq!(params.output.sample_rate, 44100);
    }
}
