pub mod params;
pub mod oscillator;
pub mod envelope;
pub mod synthesizer;
pub mod wav;

use params::{parse_params, SfxParams};

/// エラー型
#[derive(Debug)]
pub enum SfxError {
    ParseError(String),
    EncodeError(String),
}

impl std::fmt::Display for SfxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SfxError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SfxError::EncodeError(msg) => write!(f, "Encode error: {}", msg),
        }
    }
}

impl std::error::Error for SfxError {}

/// パラメータ JSON バイト列から PCM サンプル列を生成
pub fn generate(params_json: &[u8]) -> Result<Vec<f32>, SfxError> {
    let params = parse_params(params_json).map_err(SfxError::ParseError)?;
    Ok(generate_from_params(&params))
}

/// パラメータ構造体から直接生成
pub fn generate_from_params(params: &SfxParams) -> Vec<f32> {
    synthesizer::generate(params)
}

/// PCM サンプル列を WAV バイト列に変換
pub fn encode_wav(samples: &[f32], sample_rate: u32, bit_depth: u16) -> Result<Vec<u8>, SfxError> {
    wav::encode_wav(samples, sample_rate, bit_depth).map_err(SfxError::EncodeError)
}

/// ショートカット: JSON → WAV バイト列
pub fn generate_wav(params_json: &[u8]) -> Result<Vec<u8>, SfxError> {
    let params = parse_params(params_json).map_err(SfxError::ParseError)?;
    let samples = synthesizer::generate(&params);
    wav::encode_wav(&samples, params.output.sample_rate, params.output.bit_depth as u16)
        .map_err(SfxError::EncodeError)
}
