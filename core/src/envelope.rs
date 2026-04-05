use crate::params::EnvelopeParams;

/// エンベロープステージ
#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
    Attack,  // 0: 0.0 → 1.0 線形上昇
    Sustain, // 1: 1.0 + punch → 1.0 減衰
    Decay,   // 2: 1.0 → 0.0 線形下降
    Finished, // 3: 0.0
}

/// Bfxr 互換エンベロープ
///
/// 変換式（bfxr-mapping.md Section 4）:
/// - attack_samples  = param^2 * 100000
/// - sustain_samples = param^2 * 100000
/// - decay_samples   = param^2 * 100000 + 10
#[derive(Debug, Clone)]
pub struct Envelope {
    lengths: [f64; 3],
    over_lengths: [f64; 3],
    stage: Stage,
    time: f64,
    sustain_punch: f64,
}

impl Envelope {
    pub fn new(params: &EnvelopeParams) -> Self {
        // Bfxr: sustainTime 最小値 0.01
        let sustain = params.sustain.max(0.01);

        let lengths = [
            params.attack * params.attack * 100_000.0,
            sustain * sustain * 100_000.0,
            params.decay * params.decay * 100_000.0 + 10.0,
        ];

        let over_lengths = [
            if lengths[0] > 0.0 { 1.0 / lengths[0] } else { 0.0 },
            1.0 / lengths[1],
            1.0 / lengths[2],
        ];

        Self {
            lengths,
            over_lengths,
            stage: Stage::Attack,
            time: 0.0,
            sustain_punch: params.sustain_punch,
        }
    }

    /// 合成ループの全サンプル数を返す
    pub fn total_samples(&self) -> usize {
        (self.lengths[0] + self.lengths[1] + self.lengths[2]) as usize
    }

    /// 1 サンプル進めて現在のボリュームを返す
    ///
    /// Bfxr の synthWave 内のエンベロープ処理に準拠:
    /// - Stage 0 (Attack):  time * overLength[0]
    /// - Stage 1 (Sustain): 1.0 + (1.0 - time * overLength[1]) * 2.0 * punch
    /// - Stage 2 (Decay):   1.0 - time * overLength[2]
    /// - Stage 3:           0.0
    pub fn tick(&mut self) -> f64 {
        // ステージ遷移チェック
        self.time += 1.0;
        let current_length = match self.stage {
            Stage::Attack => self.lengths[0],
            Stage::Sustain => self.lengths[1],
            Stage::Decay => self.lengths[2],
            Stage::Finished => return 0.0,
        };

        if self.time > current_length {
            self.time = 0.0;
            self.stage = match self.stage {
                Stage::Attack => Stage::Sustain,
                Stage::Sustain => Stage::Decay,
                Stage::Decay => Stage::Finished,
                Stage::Finished => Stage::Finished,
            };
        }

        // ボリューム計算
        match self.stage {
            Stage::Attack => {
                if self.lengths[0] > 0.0 {
                    self.time * self.over_lengths[0]
                } else {
                    1.0
                }
            }
            Stage::Sustain => {
                1.0 + (1.0 - self.time * self.over_lengths[1]) * 2.0 * self.sustain_punch
            }
            Stage::Decay => {
                1.0 - self.time * self.over_lengths[2]
            }
            Stage::Finished => 0.0,
        }
    }

    /// エンベロープが終了したかどうか
    pub fn is_finished(&self) -> bool {
        self.stage == Stage::Finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_attack_starts_at_sustain() {
        let params = EnvelopeParams {
            attack: 0.0,
            sustain: 0.1,
            sustain_punch: 0.0,
            decay: 0.1,
        };
        let mut env = Envelope::new(&params);
        // attack=0 なので最初の tick で sustain に遷移
        let vol = env.tick();
        // sustain_punch=0 なので volume ≈ 1.0
        assert!((vol - 1.0).abs() < 0.01, "expected ~1.0, got {}", vol);
    }

    #[test]
    fn sustain_punch_boost() {
        let params = EnvelopeParams {
            attack: 0.0,
            sustain: 0.1,
            sustain_punch: 0.5,
            decay: 0.1,
        };
        let mut env = Envelope::new(&params);
        let vol = env.tick();
        // sustain 開始時: 1.0 + (1.0 - 0) * 2.0 * 0.5 = 2.0
        assert!(vol > 1.0, "punch should boost above 1.0, got {}", vol);
    }

    #[test]
    fn decay_reaches_zero() {
        let params = EnvelopeParams {
            attack: 0.0,
            sustain: 0.01,
            sustain_punch: 0.0,
            decay: 0.01,
        };
        let mut env = Envelope::new(&params);
        let total = env.total_samples();
        for _ in 0..total + 100 {
            env.tick();
        }
        assert!(env.is_finished());
        assert!((env.tick()).abs() < 1e-10);
    }

    #[test]
    fn attack_ramp_up() {
        let params = EnvelopeParams {
            attack: 0.1, // 0.1^2 * 100000 = 1000 samples
            sustain: 0.1,
            sustain_punch: 0.0,
            decay: 0.1,
        };
        let mut env = Envelope::new(&params);
        let first = env.tick();
        assert!(first < 0.01, "attack should start near 0, got {}", first);

        // tick halfway through attack
        for _ in 0..499 {
            env.tick();
        }
        let mid = env.tick();
        assert!((mid - 0.5).abs() < 0.01, "mid-attack should be ~0.5, got {}", mid);
    }

    #[test]
    fn total_samples_correct() {
        let params = EnvelopeParams {
            attack: 0.1,
            sustain: 0.2,
            sustain_punch: 0.0,
            decay: 0.3,
        };
        let env = Envelope::new(&params);
        // attack: 0.01*100000=1000, sustain: 0.04*100000=4000, decay: 0.09*100000+10=9010
        assert_eq!(env.total_samples(), 14010);
    }
}
