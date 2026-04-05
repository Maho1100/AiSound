# Bfxr パラメータマッピング

参照ソース: https://github.com/increpare/bfxr（AS3 実装）

## 周波数変換式（最重要）

```
実周波数(Hz) = (base_freq ^ 2) × 8 × 100 + 100
```

- base_freq = 0.0 → 100 Hz
- base_freq = 0.5 → 2100 Hz
- base_freq = 1.0 → 8100 Hz

この二乗カーブが Bfxr 特有の音域感の源泉。必ず正確に実装すること。

## エンベロープ長（近似）

```
total_ms ≈ attack^3 × 100 + sustain^2 × 300 + decay^2 × 300
```

## パラメータ対応表

| AiSound フィールド | Bfxr AS3 変数名 | 備考 |
|---|---|---|
| wave.type | _waveType | TODO: AS3 での enum 値を確認 |
| wave.duty_cycle | _squareDuty | TODO |
| wave.duty_sweep | _dutySweep | TODO |
| envelope.attack | _attackTime | TODO |
| envelope.sustain | _sustainTime | TODO |
| envelope.sustain_punch | _sustainPunch | TODO |
| envelope.decay | _decayTime | TODO |
| frequency.base | _startFrequency | TODO |
| frequency.limit | _minFrequency | TODO |
| frequency.slide | _slide | TODO |
| frequency.delta_slide | _deltaSlide | TODO |
| frequency.vibrato_depth | _vibratoDepth | TODO |
| frequency.vibrato_speed | _vibratoSpeed | TODO |
| filter.cutoff | _lpFilterCutoff | TODO |
| filter.cutoff_sweep | _lpFilterCutoffSweep | TODO |
| filter.resonance | _lpFilterResonance | TODO |
| filter.highpass_cutoff | _hpFilterCutoff | TODO |
| filter.highpass_sweep | _hpFilterCutoffSweep | TODO |
| phaser.offset | _phaserOffset | TODO |
| phaser.sweep | _phaserSweep | TODO |
| retrigger.repeat_speed | _repeatSpeed | TODO |
| distortion.gain | _overdriveAmount | TODO |
| distortion.compress_ratio | _compressionAmount | TODO |
| bitcrusher.bit_depth | _bitCrush | TODO |
| bitcrusher.sample_rate_reduction | _bitCrushSweep | TODO |

## フィルタ係数

TODO: Bfxr AS3 ソースの Synth.as から LPF / HPF の係数計算式を抽出して記録する。
Bfxr は独自の簡易実装を使っており、一般的な Biquad フィルタとは係数が異なる点に注意。

## TODO

- [ ] Bfxr AS3 ソースの Synth.as を精読し、上記の TODO を全て埋める
- [ ] フィルタ係数の計算式を抽出
- [ ] generateBlip() 関数の処理フローを確認
- [ ] AS3 と Rust での浮動小数点演算の差異を検証
