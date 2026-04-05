# Bfxr パラメータマッピング

参照ソース: https://github.com/increpare/bfxr（AS3 実装）
ソースファイル: `src/com/increpare/bfxr/synthesis/Synthesizer/SfxrSynth.as`（合成エンジン）
パラメータ定義: `src/com/increpare/bfxr/synthesis/Synthesizer/SfxrParams.as`（パラメータ名・範囲・デフォルト）

---

## 1. 波形タイプ列挙値

SfxrParams.as では `WAVETYPECOUNT = 12` と定義されている。
SfxrSynth.as の synthWave() 内 switch 文で使われる整数値と波形の対応:

| 整数値 | 波形名 | 説明 |
|---|---|---|
| 0 | Square | 矩形波。duty_cycle で波形比率を制御 |
| 1 | Sawtooth | 鋸歯状波 |
| 2 | Sine | 正弦波（高速近似アルゴリズム） |
| 3 | Noise (White) | ホワイトノイズ（32 サンプルバッファ） |
| 4 | Triangle | 三角波 |
| 5 | Pink Noise | ピンクノイズ（PinkNumber クラス使用） |
| 6 | Tan | tan 関数ベースの波形 |
| 7 | Whistle | 正弦波 75% + 20 倍高調波 25% の合成 |
| 8 | Breaker | `abs(1 - amp^2 * 2) - 1` の波形 |
| 9 | Bitnoise | SN76489 互換 1bit 周期ノイズ（BBC Micro 互換） |
| 10 | Randomized | `Math.floor(_phase/4) % 10` で波形 0-9 を動的に切替 |
| 11 | Buzz | SN76489 ベースだがタップ位置変更（bit3 XOR bit0） |

各波形の生成コード（synthWave 内 switch 文より）:

```actionscript
// Case 0: Square wave
_sample += overtonestrength*(((tempphase / _periodTemp) < _squareDuty) ? 0.5 : -0.5);

// Case 1: Saw wave
_sample += overtonestrength*(1.0 - (tempphase / _periodTemp) * 2.0);

// Case 2: Sine wave (fast and accurate approx)
_pos = tempphase / _periodTemp;
_pos = _pos > 0.5 ? (_pos - 1.0) * 6.28318531 : _pos * 6.28318531;
var _tempsample:Number = _pos < 0 ? 1.27323954 * _pos + .405284735 * _pos * _pos
                                   : 1.27323954 * _pos - 0.405284735 * _pos * _pos;
_sample += overtonestrength*(_tempsample < 0
    ? .225 * (_tempsample *-_tempsample - _tempsample) + _tempsample
    : .225 * (_tempsample * _tempsample - _tempsample) + _tempsample);

// Case 3: Noise (white)
_sample += overtonestrength*(_noiseBuffer[uint(tempphase * 32 / int(_periodTemp))%32]);

// Case 4: Triangle
_sample += overtonestrength*(Math.abs(1-(tempphase / _periodTemp)*2)-1);

// Case 5: Pink Noise
_sample += overtonestrength*(_pinkNoiseBuffer[uint(tempphase * 32 / int(_periodTemp))%32]);

// Case 6: Tan
_sample += Math.tan(Math.PI*tempphase/_periodTemp)*overtonestrength;

// Case 7: Whistle
// 正弦波 * 0.75 + 20 倍高調波 * 0.25（コード省略、上記 Sine と同じ近似式を 2 回適用）

// Case 8: Breaker
var amp:Number = tempphase/_periodTemp;
_sample += overtonestrength*(Math.abs(1-amp*amp*2)-1);

// Case 9: Bitnoise
_sample += overtonestrength*_oneBitNoise;

// Case 10: Randomized (switch 前の分岐)
if (wtype==10) { wtype = Math.floor(_phase/4) % 10; }

// Case 11: Buzz
_sample += overtonestrength*_buzz;
```

---

## 2. パラメータ対応表

SfxrParams.as の ParamData 配列から抽出した正式なパラメータ名と、reset() での内部変数マッピング。

| AiSound フィールド | Bfxr ParamData 名 | Bfxr 内部変数名 | 範囲 | デフォルト | 備考 |
|---|---|---|---|---|---|
| wave.type | `waveType` | `_waveType` | 0 - 11 | 2 (sine) | uint にキャスト |
| wave.duty_cycle | `squareDuty` | `_squareDuty` | 0.0 - 1.0 | 0.0 | waveType==0 のみ有効。内部値は `0.5 - param * 0.5` |
| wave.duty_sweep | `dutySweep` | `_dutySweep` | -1.0 - 1.0 | 0.0 | 内部値は `-param * 0.00005` |
| envelope.attack | `attackTime` | `_envelopeLength0` | 0.0 - 1.0 | 0.0 | サンプル数 = `param^2 * 100000` |
| envelope.sustain | `sustainTime` | `_envelopeLength1` | 0.0 - 1.0 | 0.3 | サンプル数 = `param^2 * 100000` |
| envelope.sustain_punch | `sustainPunch` | `_sustainPunch` | 0.0 - 1.0 | 0.0 | サステイン開始時に `1.0 + (1.0 - t) * 2.0 * punch` |
| envelope.decay | `decayTime` | `_envelopeLength2` | 0.0 - 1.0 | 0.4 | サンプル数 = `param^2 * 100000 + 10` |
| frequency.base | `startFrequency` | `_period` | 0.0 - 1.0 | 0.3 | `_period = 100.0 / (param^2 + 0.001)` |
| frequency.limit | `minFrequency` | `_maxPeriod` | 0.0 - 1.0 | 0.0 | `_maxPeriod = 100.0 / (param^2 + 0.001)` |
| frequency.slide | `slide` | `_slide` | -1.0 - 1.0 | 0.0 | `_slide = 1.0 - param^3 * 0.01` |
| frequency.delta_slide | `deltaSlide` | `_deltaSlide` | -1.0 - 1.0 | 0.0 | `_deltaSlide = -param^3 * 0.000001` |
| frequency.vibrato_depth | `vibratoDepth` | `_vibratoAmplitude` | 0.0 - 1.0 | 0.0 | `_vibratoAmplitude = param * 0.5` |
| frequency.vibrato_speed | `vibratoSpeed` | `_vibratoSpeed` | 0.0 - 1.0 | 0.0 | `_vibratoSpeed = param^2 * 0.01` |
| frequency.vibrato_delay | (該当なし) | (該当なし) | - | - | **Bfxr に存在しない。AiSound 独自拡張** |
| arpeggio.multiplier | `changeAmount` | `_changeAmount` | -1.0 - 1.0 | 0.0 | 後述 (Section 8) |
| arpeggio.speed | `changeSpeed` | `_changeLimit` | 0.0 - 1.0 | 0.0 | 後述 (Section 8) |
| arpeggio.limit | `changeRepeat` | `_changePeriod` | 0.0 - 1.0 | 0.0 | 後述 (Section 8) |
| filter.cutoff | `lpFilterCutoff` | `_lpFilterCutoff` | 0.0 - 1.0 | 1.0 | `_lpFilterCutoff = param^3 * 0.1` |
| filter.cutoff_sweep | `lpFilterCutoffSweep` | `_lpFilterDeltaCutoff` | -1.0 - 1.0 | 0.0 | `_lpFilterDeltaCutoff = 1.0 + param * 0.0001` |
| filter.resonance | `lpFilterResonance` | `_lpFilterDamping` | 0.0 - 1.0 | 0.0 | 後述 (Section 5) |
| filter.highpass_cutoff | `hpFilterCutoff` | `_hpFilterCutoff` | 0.0 - 1.0 | 0.0 | `_hpFilterCutoff = param^2 * 0.1` |
| filter.highpass_sweep | `hpFilterCutoffSweep` | `_hpFilterDeltaCutoff` | -1.0 - 1.0 | 0.0 | `_hpFilterDeltaCutoff = 1.0 + param * 0.0003` |
| phaser.offset | `flangerOffset` | `_flangerOffset` | -1.0 - 1.0 | 0.0 | 後述 (Section 6)。名称注意: Bfxr では「flanger」 |
| phaser.sweep | `flangerSweep` | `_flangerDeltaOffset` | -1.0 - 1.0 | 0.0 | 後述 (Section 6) |
| retrigger.repeat_speed | `repeatSpeed` | `_repeatLimit` | 0.0 - 1.0 | 0.0 | `_repeatLimit = (1-param)^2 * 20000 + 32`。0.0 で無効 |
| distortion.gain | (該当なし) | (該当なし) | - | - | **Bfxr に存在しない。AiSound 独自拡張** |
| distortion.compress_ratio | `compressionAmount` | `_compression_factor` | 0.0 - 1.0 | 0.3 | `_compression_factor = 1 / (1 + 4 * param)` |
| bitcrusher.bit_depth | `bitCrush` | `_bitcrush_freq` | 0.0 - 1.0 | 0.0 | `_bitcrush_freq = 1 - param^(1/3)` |
| bitcrusher.sample_rate_reduction | `bitCrushSweep` | `_bitcrush_freq_sweep` | -1.0 - 1.0 | 0.0 | `_bitcrush_freq_sweep = -param * 0.000015` |

### Bfxr にあって AiSound スキーマに無いパラメータ

| Bfxr ParamData 名 | 内部変数 | 説明 |
|---|---|---|
| `masterVolume` | `_masterVolume` | 全体音量 (`param^2`) |
| `overtones` | `_overtones` | ハーモニクス数 (`param * 10` → 整数) |
| `overtoneFalloff` | `_overtoneFalloff` | 高調波の減衰率 |
| `changeAmount2` | `_changeAmount2` | 第 2 ピッチジャンプ量 |
| `changeSpeed2` | `_changeLimit2` | 第 2 ピッチジャンプ速度 |

---

## 3. 周波数変換式（ソース検証済み）

### 設計書の記述

```
実周波数(Hz) = (base_freq ^ 2) * 8 * 100 + 100
```

### 実際の Bfxr ソースコード（reset 関数）

```actionscript
_period = 100.0 / (p.getParam("startFrequency") * p.getParam("startFrequency") + 0.001);
```

**重要**: Bfxr は周波数を Hz ではなく「周期」（サンプル数）で管理している。
設計書の `Hz = (f^2) * 800 + 100` は**不正確**。

### 正確な変換式

Bfxr の内部サンプルレートは 44100 Hz * 8 = 352800 Hz（8x supersampling）。

```
_period = 100.0 / (startFrequency^2 + 0.001)
```

周期からHz への変換:
```
Hz = sampleRate_internal / _period
   = 352800 / (100.0 / (f^2 + 0.001))
   = 352800 * (f^2 + 0.001) / 100.0
   = 3528 * (f^2 + 0.001)
```

具体的な値:
- f = 0.0 → `_period = 100000` → Hz = 3528 * 0.001 = 3.5 Hz
- f = 0.3 → `_period = 1098.9` → Hz = 3528 * 0.091 = 321 Hz
- f = 0.5 → `_period = 396.8`  → Hz = 3528 * 0.251 = 886 Hz
- f = 1.0 → `_period = 99.9`   → Hz = 3528 * 1.001 = 3531 Hz

**注意**: `_periodTemp` は整数に切り捨てられ、最小値は 8 に制限される。

```actionscript
_periodTemp = int(_periodTemp);
if(_periodTemp < 8) _periodTemp = 8;
```

---

## 4. エンベロープ長計算（ソース検証済み）

### 設計書の記述

```
total_ms ≈ attack^3 * 100 + sustain^2 * 300 + decay^2 * 300
```

### 実際の Bfxr ソースコード（reset 関数）

```actionscript
_envelopeLength0 = p.getParam("attackTime") * p.getParam("attackTime") * 100000.0;
_envelopeLength1 = p.getParam("sustainTime") * p.getParam("sustainTime") * 100000.0;
_envelopeLength2 = p.getParam("decayTime") * p.getParam("decayTime") * 100000.0 + 10;
_envelopeLength = _envelopeLength0;
_envelopeFullLength = _envelopeLength0 + _envelopeLength1 + _envelopeLength2;

_envelopeOverLength0 = 1.0 / _envelopeLength0;
_envelopeOverLength1 = 1.0 / _envelopeLength1;
_envelopeOverLength2 = 1.0 / _envelopeLength2;
```

### 正確な変換式

全て **サンプル数**（合成ループの反復回数）で管理される:

```
attack_samples  = attackTime^2  * 100000
sustain_samples = sustainTime^2 * 100000
decay_samples   = decayTime^2   * 100000 + 10
```

ミリ秒への変換（サンプルレート 44100 Hz）:
```
attack_ms  = attackTime^2  * 100000 / 44100 * 1000 = attackTime^2  * 2267.6
sustain_ms = sustainTime^2 * 100000 / 44100 * 1000 = sustainTime^2 * 2267.6
decay_ms   = (decayTime^2  * 100000 + 10) / 44100 * 1000
```

**設計書の近似式は不正確**: 全て `param^2 * 100000` サンプルであり、`param^3` ではない。

### 最小長制限

```actionscript
public static const MIN_LENGTH:Number = 0.18;

private function clampTotalLength():void {
    var totalTime:Number = p.getParam("attackTime") + p.getParam("sustainTime") + p.getParam("decayTime");
    if (totalTime < MIN_LENGTH) {
        var multiplier:Number = MIN_LENGTH / totalTime;
        p.setParam("attackTime", p.getParam("attackTime") * multiplier);
        p.setParam("sustainTime", p.getParam("sustainTime") * multiplier);
        p.setParam("decayTime", p.getParam("decayTime") * multiplier);
    }
}
```

また、sustainTime は最小 0.01 に制限される:

```actionscript
if (p.getParam("sustainTime") < 0.01) p.setParam("sustainTime", 0.01);
```

### エンベロープ適用コード（synthWave 内）

```actionscript
// ステージ遷移
if(++_envelopeTime > _envelopeLength) {
    _envelopeTime = 0;
    switch(++_envelopeStage) {
        case 1: _envelopeLength = _envelopeLength1; break;
        case 2: _envelopeLength = _envelopeLength2; break;
    }
}

// ボリューム計算
switch(_envelopeStage) {
    case 0: _envelopeVolume = _envelopeTime * _envelopeOverLength0;                           break;
    case 1: _envelopeVolume = 1.0 + (1.0 - _envelopeTime * _envelopeOverLength1) * 2.0 * _sustainPunch; break;
    case 2: _envelopeVolume = 1.0 - _envelopeTime * _envelopeOverLength2;                     break;
    case 3: _envelopeVolume = 0.0; _finished = true;                                          break;
}
```

ステージの意味:
- **Stage 0 (Attack)**: 0.0 から 1.0 へ線形上昇
- **Stage 1 (Sustain)**: `1.0 + (1.0 - t) * 2.0 * sustainPunch` で開始し 1.0 へ減衰
- **Stage 2 (Decay)**: 1.0 から 0.0 へ線形下降
- **Stage 3**: 0.0 で終了フラグ

---

## 5. フィルタ係数（ソース検証済み）

### LPF 係数初期化（reset 関数）

```actionscript
_lpFilterPos = 0.0;
_lpFilterDeltaPos = 0.0;
_lpFilterCutoff = p.getParam("lpFilterCutoff") * p.getParam("lpFilterCutoff") * p.getParam("lpFilterCutoff") * 0.1;
_lpFilterDeltaCutoff = 1.0 + p.getParam("lpFilterCutoffSweep") * 0.0001;
_lpFilterDamping = 5.0 / (1.0 + p.getParam("lpFilterResonance") * p.getParam("lpFilterResonance") * 20.0)
                   * (0.01 + _lpFilterCutoff);
if (_lpFilterDamping > 0.8) _lpFilterDamping = 0.8;
_lpFilterDamping = 1.0 - _lpFilterDamping;
_lpFilterOn = p.getParam("lpFilterCutoff") != 1.0;
```

### HPF 係数初期化（reset 関数）

```actionscript
_hpFilterPos = 0.0;
_hpFilterCutoff = p.getParam("hpFilterCutoff") * p.getParam("hpFilterCutoff") * 0.1;
_hpFilterDeltaCutoff = 1.0 + p.getParam("hpFilterCutoffSweep") * 0.0003;
```

### フィルタ有効判定

```actionscript
_filters = p.getParam("lpFilterCutoff") != 1.0 || p.getParam("hpFilterCutoff") != 0.0;
```

### サンプル単位のフィルタ処理（synthWave 内、8x supersampling ループ内）

```actionscript
if (_filters)
{
    _lpFilterOldPos = _lpFilterPos;
    _lpFilterCutoff *= _lpFilterDeltaCutoff;
         if(_lpFilterCutoff < 0.0) _lpFilterCutoff = 0.0;
    else if(_lpFilterCutoff > 0.1) _lpFilterCutoff = 0.1;

    if(_lpFilterOn)
    {
        _lpFilterDeltaPos += (_sample - _lpFilterPos) * _lpFilterCutoff;
        _lpFilterDeltaPos *= _lpFilterDamping;
    }
    else
    {
        _lpFilterPos = _sample;
        _lpFilterDeltaPos = 0.0;
    }

    _lpFilterPos += _lpFilterDeltaPos;

    _hpFilterPos += _lpFilterPos - _lpFilterOldPos;
    _hpFilterPos *= 1.0 - _hpFilterCutoff;
    _sample = _hpFilterPos;
}
```

### フィルタの動作解説

**LPF（非標準の 2 変数実装）**:
- `_lpFilterDeltaPos` は「速度」変数（damped velocity）
- `_lpFilterPos` は「位置」変数（filtered output）
- 式を展開すると:
  ```
  velocity += (input - position) * cutoff
  velocity *= damping
  position += velocity
  ```
- これはバネ-ダンパー系のシミュレーションに相当する
- 一般的な 1 次 IIR フィルタではなく、2 次的な挙動を持つ

**HPF（DC ブロッカー型）**:
- LPF 出力の差分を積算し、リーク係数で減衰
- 式:
  ```
  hpPos += lpPos_new - lpPos_old
  hpPos *= (1.0 - hpCutoff)
  output = hpPos
  ```

**カットオフのスイープ**: 毎サンプル `cutoff *= deltaCutoff` で乗算更新される。
- LPF: `_lpFilterDeltaCutoff = 1.0 + param * 0.0001`
- HPF: `_hpFilterDeltaCutoff = 1.0 + param * 0.0003`

**HPF カットオフの毎サンプル更新**（supersampling ループの外、メインループ内）:

```actionscript
if(_filters && _hpFilterDeltaCutoff != 0.0) {
    _hpFilterCutoff *= _hpFilterDeltaCutoff;
         if(_hpFilterCutoff < 0.00001) _hpFilterCutoff = 0.00001;
    else if(_hpFilterCutoff > 0.1)     _hpFilterCutoff = 0.1;
}
```

---

## 6. フェイザー（フランジャー）実装（ソース検証済み）

**注意**: Bfxr では「flanger」と呼んでいるが、AiSound 設計書では「phaser」に対応。

### 初期化（reset 関数）

```actionscript
_flanger = p.getParam("flangerOffset") != 0.0 || p.getParam("flangerSweep") != 0.0;

_flangerOffset = p.getParam("flangerOffset") * p.getParam("flangerOffset") * 1020.0;
if(p.getParam("flangerOffset") < 0.0) _flangerOffset = -_flangerOffset;
_flangerDeltaOffset = p.getParam("flangerSweep") * p.getParam("flangerSweep") * p.getParam("flangerSweep") * 0.2;
_flangerPos = 0;

if(!_flangerBuffer) _flangerBuffer = new Vector.<Number>(1024, true);
for(var i:uint = 0; i < 1024; i++) _flangerBuffer[i] = 0.0;
```

### サンプル処理（synthWave 内）

オフセット更新（メインループ内、supersampling 外）:
```actionscript
if (_flanger) {
    _flangerOffset += _flangerDeltaOffset;
    _flangerInt = int(_flangerOffset);
         if(_flangerInt < 0)    _flangerInt = -_flangerInt;
    else if (_flangerInt > 1023) _flangerInt = 1023;
}
```

サンプル加算（supersampling ループ内、フィルタ後）:
```actionscript
if (_flanger) {
    _flangerBuffer[_flangerPos&1023] = _sample;
    _sample += _flangerBuffer[(_flangerPos - _flangerInt + 1024) & 1023];
    _flangerPos = (_flangerPos + 1) & 1023;
}
```

### フェイザーの動作解説

- **バッファサイズ**: 1024 サンプル（リングバッファ）
- **オフセット計算**: `param^2 * 1020.0`（符号保持）
- **スイープ計算**: `param^3 * 0.2`
- **オフセットのクランプ**: 0 - 1023 の範囲
- **動作**: 現在のサンプルをバッファに書き込み、_flangerInt サンプル前のバッファ値を加算

---

## 7. メイン合成ループ（synthWave）処理順序

サンプルレート: 44100 Hz（出力）、8x supersampling で内部 352800 Hz 相当。

### 外側ループ（1 出力サンプルあたり 1 回）

```
1. 終了チェック
2. リピート処理: _repeatLimit に達したら reset(false) でパラメータ部分リセット
3. ピッチジャンプ周期管理: _changePeriodTime のインクリメントと周期リセット
4. ピッチジャンプ 1: _changeLimit に達したら _period *= _changeAmount
5. ピッチジャンプ 2: _changeLimit2 に達したら _period *= _changeAmount2
6. 周波数スライド: _slide += _deltaSlide; _period *= _slide
7. 周波数下限チェック: _period > _maxPeriod ならクランプ、ミュート判定
8. ビブラート: _periodTemp = _period * (1.0 + sin(_vibratoPhase) * _vibratoAmplitude)
9. 周期の整数化と最小値制限: _periodTemp = int(_periodTemp); min 8
10. デューティスイープ: _squareDuty += _dutySweep (wave==0 のみ)
11. エンベロープ更新: ステージ遷移とボリューム計算
12. フランジャーオフセット更新
13. HPF カットオフスイープ
```

### 内側ループ（8x supersampling、1 出力サンプルあたり 8 回）

```
14. 位相進行: _phase++、周期越えで位相リセット + ノイズバッファ再生成
15. オーバートーン付き波形生成: k = 0 .. _overtones
    - tempphase = (_phase * (k+1)) % _periodTemp
    - switch(wtype) で波形サンプル生成
    - overtonestrength *= (1 - _overtoneFalloff)
16. LPF/HPF フィルタ適用
17. フランジャー適用
18. _superSample に加算
```

### 後処理（外側ループ続き）

```
19. クリッピング: _superSample を -8.0 .. 8.0 に制限
20. 音量適用: _superSample = _masterVolume * _envelopeVolume * _superSample * 0.125
21. ビットクラッシュ: _bitcrush_phase による sample-and-hold
22. コンプレッション: pow(abs(sample), _compression_factor) * sign
23. ミュート適用
24. バッファ書き出し
```

---

## 8. アルペジオ（ピッチジャンプ）実装

Bfxr は 2 段階のピッチジャンプと繰り返しを持つ。AiSound の `arpeggio` グループに対応。

### パラメータ変換（reset 関数）

```actionscript
// ピッチジャンプ量（Amount）
if (p.getParam("changeAmount") > 0.0)
    _changeAmount = 1.0 - p.getParam("changeAmount") * p.getParam("changeAmount") * 0.9;
else
    _changeAmount = 1.0 + p.getParam("changeAmount") * p.getParam("changeAmount") * 10.0;

// ピッチジャンプ速度（Speed）
if(p.getParam("changeSpeed") == 1.0) _changeLimit = 0;
else _changeLimit = (1.0 - p.getParam("changeSpeed")) * (1.0 - p.getParam("changeSpeed")) * 20000 + 32;

// 繰り返し周期（Repeat）
_changePeriod = Math.max(((1-p.getParam("changeRepeat"))+0.1)/1.1) * 20000 + 32;

// changeLimit は changeRepeat で縮小される
_changeLimit *= (1-p.getParam("changeRepeat")+0.1)/1.1;
```

### synthWave 内での適用

```actionscript
// 周期リセット
_changePeriodTime++;
if (_changePeriodTime >= _changePeriod) {
    _changeTime = 0;
    _changeTime2 = 0;
    _changePeriodTime = 0;
    if (_changeReached) {
        _period /= _changeAmount;   // 前回のジャンプを巻き戻す
        _changeReached = false;
    }
    if (_changeReached2) {
        _period /= _changeAmount2;
        _changeReached2 = false;
    }
}

// ジャンプ 1 適用
if(!_changeReached) {
    if(++_changeTime >= _changeLimit) {
        _changeReached = true;
        _period *= _changeAmount;
    }
}

// ジャンプ 2 適用
if(!_changeReached2) {
    if(++_changeTime2 >= _changeLimit2) {
        _period *= _changeAmount2;
        _changeReached2 = true;
    }
}
```

### AiSound マッピング

| AiSound | Bfxr パラメータ | 説明 |
|---|---|---|
| arpeggio.multiplier | changeAmount | 正: `1.0 - val^2 * 0.9`（周波数低下）、負: `1.0 + val^2 * 10.0`（周波数上昇） |
| arpeggio.speed | changeSpeed | `(1-val)^2 * 20000 + 32` サンプル後にジャンプ |
| arpeggio.limit | changeRepeat | ジャンプの繰り返し周期。0 で繰り返しなし |

**注意**: Bfxr は 2 段階のピッチジャンプ（changeAmount / changeAmount2）を持つが、AiSound v1 スキーマでは 1 段階のみ。将来の拡張候補。

---

## 9. ビットクラッシャー実装

### 初期化（reset 関数）

```actionscript
_bitcrush_freq = 1 - Math.pow(p.getParam("bitCrush"), 1.0/3.0);
_bitcrush_freq_sweep = -p.getParam("bitCrushSweep") * 0.000015;
_bitcrush_phase = 0;
_bitcrush_last = 0;
```

### サンプル処理（synthWave 内、エンベロープ適用後）

```actionscript
_bitcrush_phase += _bitcrush_freq;
if (_bitcrush_phase > 1) {
    _bitcrush_phase = 0;
    _bitcrush_last = _superSample;
}
_bitcrush_freq = Math.max(Math.min(_bitcrush_freq + _bitcrush_freq_sweep, 1), 0);
_superSample = _bitcrush_last;
```

### 動作解説

- Bfxr の「Bit Crush」は実際にはサンプルレート削減（sample-and-hold）
- `_bitcrush_freq` が 1.0 に近いほど更新頻度が高い（劣化少ない）
- `_bitcrush_freq` が 0.0 に近いほど更新が遅い（劣化大きい）
- bitCrush パラメータが 0.0 → `_bitcrush_freq = 1.0`（劣化なし）
- bitCrush パラメータが 1.0 → `_bitcrush_freq = 0.0`（最大劣化）

**注意**: ビット深度の削減は Bfxr にはない。AiSound の `bitcrusher.bit_depth` は独自拡張。

---

## 10. コンプレッション実装

### 初期化

```actionscript
_compression_factor = 1 / (1 + 4 * p.getParam("compressionAmount"));
```

### サンプル処理

```actionscript
if (_superSample > 0) {
    _superSample = Math.pow(_superSample, _compression_factor);
} else {
    _superSample = -Math.pow(-_superSample, _compression_factor);
}
```

- compressionAmount = 0.0 → factor = 1.0（変化なし）
- compressionAmount = 0.3 → factor = 0.4545...（中程度の圧縮）
- compressionAmount = 1.0 → factor = 0.2（強い圧縮）
- factor < 1.0 のとき `pow(x, factor)` は小さな値を持ち上げ、動的レンジを圧縮する

---

## 11. リピート（Retrigger）実装

### 初期化

```actionscript
if (p.getParam("repeatSpeed") == 0.0) _repeatLimit = 0;
else _repeatLimit = int((1.0 - p.getParam("repeatSpeed")) * (1.0 - p.getParam("repeatSpeed")) * 20000) + 32;
```

### サンプル処理

```actionscript
if(_repeatLimit != 0) {
    if(++_repeatTime >= _repeatLimit) {
        _repeatTime = 0;
        reset(false);   // 部分リセット（エンベロープ・フィルタ等は totalReset=false で維持されない）
    }
}
```

- `reset(false)` は周波数・スライド・ピッチジャンプ関連のみリセットする
- エンベロープやフィルタ状態は `totalReset=true` の場合のみ初期化される

---

## 12. 周波数スライド実装

### 初期化

```actionscript
_slide = 1.0 - p.getParam("slide") * p.getParam("slide") * p.getParam("slide") * 0.01;
_deltaSlide = -p.getParam("deltaSlide") * p.getParam("deltaSlide") * p.getParam("deltaSlide") * 0.000001;
```

### サンプル処理

```actionscript
_slide += _deltaSlide;
_period *= _slide;

if(_period > _maxPeriod) {
    _period = _maxPeriod;
    if(_minFreqency > 0.0) {
        _muted = true;
    }
}
```

- `_slide` は毎サンプル `_period` に乗算される
- `_slide < 1.0` → 周期が縮小 → 周波数上昇
- `_slide > 1.0` → 周期が増大 → 周波数下降
- `_deltaSlide` は `_slide` 自体を毎サンプル変化させる（加速度）

---

## 13. 設計書との差異まとめ

| 項目 | 設計書の記述 | 実際のソースコード | 影響 |
|---|---|---|---|
| 周波数変換 | `Hz = f^2 * 800 + 100` | `_period = 100.0 / (f^2 + 0.001)` → Hz は内部サンプルレート依存 | **重大**: 設計書の式は不正確。実装時は period ベースで行うこと |
| エンベロープ長 | `attack^3 * 100` | `attackTime^2 * 100000` サンプル | **重大**: 指数が 2 であり 3 ではない。係数も異なる |
| phaser.offset | `_phaserOffset` | `_flangerOffset`（Bfxr では flanger と命名） | 軽微: 名称の違いのみ |
| phaser.sweep | `_phaserSweep` | `_flangerDeltaOffset`（内部変数名が異なる） | 軽微 |
| distortion.gain | `_overdriveAmount` | 該当パラメータなし | **重大**: Bfxr にオーバードライブは存在しない。AiSound 独自拡張として実装する |
| frequency.vibrato_delay | 設計書に存在 | Bfxr に該当なし | 軽微: AiSound 独自拡張 |
| bitcrusher.bit_depth | ビット深度削減 | サンプルレート削減（sample-and-hold） | **重大**: Bfxr の bitCrush はビット深度ではなくサンプルレート削減 |

---

## 14. ノイズバッファ実装

### 初期化（reset 関数）

```actionscript
if(!_noiseBuffer) _noiseBuffer = new Vector.<Number>(32, true);
if(!_pinkNoiseBuffer) _pinkNoiseBuffer = new Vector.<Number>(32, true);
if(!_loResNoiseBuffer) _loResNoiseBuffer = new Vector.<Number>(32, true);

for(i = 0; i < 32; i++) _noiseBuffer[i] = Math.random() * 2.0 - 1.0;
for(i = 0; i < 32; i++) _pinkNoiseBuffer[i] = _pinkNumber.GetNextValue();
for(i = 0; i < 32; i++) _loResNoiseBuffer[i] = ((i%LoResNoisePeriod)==0)
    ? Math.random()*2.0-1.0
    : _loResNoiseBuffer[i-1];

// Bitnoise (waveType 9) - SN76489 互換
_oneBitNoiseState = 1 << 14;
_oneBitNoise = 0;

// Buzz (waveType 11)
_buzzState = 1 << 14;
_buzz = 0;
```

### 周期ごとの再生成（synthWave 内、phase リセット時）

```actionscript
if(_waveType == 3) {
    for(var n:uint = 0; n < 32; n++) _noiseBuffer[n] = Math.random() * 2.0 - 1.0;
} else if (_waveType == 5) {
    for(n = 0; n < 32; n++) _pinkNoiseBuffer[n] = _pinkNumber.GetNextValue();
} else if (_waveType == 6) {
    for(n = 0; n < 32; n++) _loResNoiseBuffer[n] = ((n%LoResNoisePeriod)==0)
        ? Math.random()*2.0-1.0 : _loResNoiseBuffer[n-1];
} else if (_waveType == 9) {
    // Bitnoise - LFSR with taps at bit 0 and bit 1
    var feedBit:int = (_oneBitNoiseState >> 1 & 1) ^ (_oneBitNoiseState & 1);
    _oneBitNoiseState = _oneBitNoiseState >> 1 | (feedBit << 14);
    _oneBitNoise = (~_oneBitNoiseState & 1) - 0.5;
} else if (_waveType == 11) {
    // Buzz - LFSR with taps at bit 0 and bit 3
    var fb:int = (_buzzState >> 3 & 1) ^ (_buzzState & 1);
    _buzzState = _buzzState >> 1 | (fb << 14);
    _buzz = (~_buzzState & 1) - 0.5;
}
```

- LoResNoisePeriod = 8（定数）
- ノイズバッファサイズ = 32（全種類共通）
- LFSR（Bitnoise / Buzz）は 15 bit シフトレジスタ

---

## 15. オーバートーン（ハーモニクス）実装

### 初期化

```actionscript
_overtones = p.getParam("overtones") * 10;  // 0-10 の整数
_overtoneFalloff = p.getParam("overtoneFalloff");
```

### 合成（synthWave 内）

```actionscript
_sample = 0;
var overtonestrength:Number = 1;
for (var k:int = 0; k <= _overtones; k++) {
    var tempphase:Number = (_phase * (k+1)) % _periodTemp;
    // switch(wtype) で波形生成...
    _sample += overtonestrength * (波形値);
    overtonestrength *= (1 - _overtoneFalloff);
}
```

- k=0 が基本波、k=1 以降が倍音
- 各倍音の位相は `_phase * (k+1)`（整数倍周波数）
- 振幅は `(1 - overtoneFalloff)^k` で指数減衰

---

## TODO

- [x] Bfxr AS3 ソースの SfxrSynth.as を精読し、パラメータ対応表を完成
- [x] フィルタ係数の計算式を抽出
- [ ] AS3 と Rust での浮動小数点演算の差異を検証
- [ ] AiSound 独自拡張パラメータ（vibrato_delay, distortion.gain, bitcrusher.bit_depth）の仕様決定
- [ ] Bfxr の generatePickupCoin() 等のプリセット生成コードを抽出（params/random 実装時に参照）
