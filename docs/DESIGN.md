# Bfxr 互換 効果音生成ツール 設計書 v1.0

プロジェクト: AiSound
作業ディレクトリ: F:\App\Ohlo\AiSound
作成日: 2026-04-05

---

## 目次

1. 全体アーキテクチャ
2. モジュール責務
3. パラメータファイル スキーマ
4. Bfxr 主要パラメータ整理
5. 生成エンジン 処理フロー
6. パラメータ生成ツールの役割
7. MVP 実装順序
8. 技術選定
9. ディレクトリ構成
10. API 案
11. テスト戦略
12. 注意点
13. GitHub 運用方針
14. 最初の 1 週間 作業リスト

---

## 1. 全体アーキテクチャ

### 基本方針

コアエンジンは I/O を持たない純粋関数として設計する。
パラメータファイル（JSON）を受け取り、PCM サンプル列を返すだけの責務とする。
将来 WebAssembly 化した際も同じ API でブラウザ上で動作する。

### レイヤー構成

```
フロントエンド層
  CLI Tool | GUI App（将来）| Web App（将来）
      ↓
  パラメータファイル (.json)
      ↓
パラメータ管理層
  preset/ | random/ | mutate/ | validator/ | serializer/
      ↓
  ValidatedParams（型保証済み内部表現）
      ↓
生成エンジン層 (core/)
  synthesizer/ | oscillator/ | envelope/ | filter/
  phaser/ | distortion/ | arpeggio/ | bitcrusher/
      ↓
  f32 PCM サンプル列
      ↓
出力層 (output/)
  wav-encoder/ | playback/ | stream/
```

---

## 2. モジュール責務

| モジュール | 責務 | 依存先 |
|---|---|---|
| core/synthesizer | 各モジュールを統合してサンプル列を生成する司令塔 | oscillator, envelope, filter, fx |
| core/oscillator | 波形生成（square / sawtooth / sine / triangle / noise / breaker） | なし |
| core/envelope | ADSR + 各種カーブの時間変化適用 | なし |
| core/filter | ローパス / ハイパスフィルタ | なし |
| core/phaser | フェイザーエフェクト | なし |
| core/arpeggio | アルペジオ（周波数ジャンプ列） | なし |
| core/bitcrusher | ビット深度・サンプルレート削減 | なし |
| params/schema | パラメータ型定義・デフォルト値 | なし |
| params/validator | 値域チェック・型変換・正規化 | schema |
| params/preset | プリセット定義・読み込み | schema |
| params/random | カテゴリ別ランダム生成・シード管理 | schema |
| params/mutate | 既存パラメータへの変異適用 | schema |
| output/wav | PCM バッファ → WAV バイト列エンコード | なし |
| output/playback | WAV バイト列のリアルタイム再生 | wav |
| cli/main | CLI 引数解析、各層の呼び出し | 全層 |

### 依存ルール

- 依存方向は常に「上位層 → 下位層」の一方向のみ
- core/ 内の各モジュールは互いに依存しない（synthesizer のみが統合する）
- params/ は core/ に依存しない（型定義のみ共有）

---

## 3. パラメータファイル スキーマ

### 設計方針

- 全フィールドは正規化済み 0.0〜1.0（Bfxr 準拠）
- 実 Hz / ms への変換はコアエンジンが行う
- 省略フィールドはデフォルト値で補完（params/validator の責務）
- version フィールドで後方互換を保証
- フィールド名は意味ベースで命名（Bfxr の AS3 変数名に引きずられない）

### スキーマ定義（schemas/sfx-params.v1.json）

```json
{
  "$schema": "https://your-project/schema/sfx-params.v1.json",
  "version": "1",
  "meta": {
    "name": "coin",
    "category": "pickup",
    "created_by": "random",
    "seed": 42,
    "created_at": "2026-04-05T00:00:00Z"
  },
  "wave": {
    "type": "square",
    "duty_cycle": 0.5,
    "duty_sweep": 0.0
  },
  "envelope": {
    "attack": 0.0,
    "sustain": 0.1,
    "sustain_punch": 0.5,
    "decay": 0.3
  },
  "frequency": {
    "base": 0.3,
    "limit": 0.0,
    "slide": 0.35,
    "delta_slide": 0.0,
    "vibrato_depth": 0.0,
    "vibrato_speed": 0.0,
    "vibrato_delay": 0.0
  },
  "arpeggio": {
    "multiplier": 0.0,
    "speed": 0.0,
    "limit": 0
  },
  "filter": {
    "cutoff": 1.0,
    "cutoff_sweep": 0.0,
    "resonance": 0.0,
    "highpass_cutoff": 0.0,
    "highpass_sweep": 0.0
  },
  "phaser": {
    "offset": 0.0,
    "sweep": 0.0
  },
  "retrigger": {
    "repeat_speed": 0.0
  },
  "distortion": {
    "gain": 0.0,
    "compress_ratio": 0.5
  },
  "bitcrusher": {
    "bit_depth": 16,
    "sample_rate_reduction": 0
  },
  "output": {
    "volume": 1.0,
    "sample_rate": 44100,
    "bit_depth": 16
  }
}
```

### meta フィールド仕様

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| name | string | 推奨 | 人間が読める名前 |
| category | string | 推奨 | pickup / laser / explosion / powerup / hit / jump / blip |
| created_by | string | 推奨 | preset / random / mutate / user |
| seed | integer | 任意 | ランダム生成時のシード値（再現用） |
| created_at | string | 任意 | ISO 8601 形式 |

### バージョン管理方針

- version は文字列で管理（"1", "2" ...）
- バージョンアップ時はマイグレーション関数を必ず用意する
- 旧バージョンのファイルは自動的に最新バージョンへ変換して読み込む

---

## 4. Bfxr 主要パラメータ整理

参照ソース: https://github.com/increpare/bfxr（AS3 実装）

### 4-1. 重要な変換式

**周波数変換（最重要）**

```
実周波数(Hz) = (base_freq ^ 2) × 8 × 100 + 100
```

- base_freq = 0.0 → 100 Hz
- base_freq = 0.5 → 2100 Hz
- base_freq = 1.0 → 8100 Hz
- この二乗カーブが Bfxr 特有の音域感の源泉。必ず正確に実装すること。

**エンベロープ長（近似）**

```
total_ms ≈ attack^3 × 100 + sustain^2 × 300 + decay^2 × 300
```

### 4-2. 波形 (Wave)

| パラメータ | 範囲 | デフォルト | 効果 |
|---|---|---|---|
| wave.type | square / sawtooth / sine / triangle / noise / breaker | square | 波形の種類 |
| wave.duty_cycle | 0.0〜1.0 | 0.5 | 矩形波のデューティ比（0.5 = 50%） |
| wave.duty_sweep | -1.0〜1.0 | 0.0 | デューティ比の時間変化速度 |

### 4-3. エンベロープ (Envelope)

| パラメータ | 範囲 | デフォルト | 効果 |
|---|---|---|---|
| envelope.attack | 0.0〜1.0 | 0.0 | アタック時間（0 = 瞬間立ち上がり） |
| envelope.sustain | 0.0〜1.0 | 0.3 | サステイン時間 |
| envelope.sustain_punch | 0.0〜1.0 | 0.0 | サステイン開始時の音量ブースト |
| envelope.decay | 0.0〜1.0 | 0.4 | ディケイ時間 |

### 4-4. 周波数 (Frequency)

| パラメータ | 範囲 | デフォルト | 効果 |
|---|---|---|---|
| frequency.base | 0.0〜1.0 | 0.3 | 基本周波数（二乗カーブ） |
| frequency.limit | 0.0〜1.0 | 0.0 | スライド下限周波数 |
| frequency.slide | -1.0〜1.0 | 0.0 | 周波数スライド速度（対数スケール） |
| frequency.delta_slide | -1.0〜1.0 | 0.0 | スライドの加速度 |
| frequency.vibrato_depth | 0.0〜1.0 | 0.0 | ビブラート深さ |
| frequency.vibrato_speed | 0.0〜1.0 | 0.0 | ビブラート速度 |
| frequency.vibrato_delay | 0.0〜1.0 | 0.0 | ビブラート開始までの遅延 |

### 4-5. アルペジオ (Arpeggio)

| パラメータ | 範囲 | デフォルト | 効果 |
|---|---|---|---|
| arpeggio.multiplier | 0.0〜1.0 | 0.0 | 周波数乗数（0.5 = オクターブ下、2.0 = オクターブ上） |
| arpeggio.speed | 0.0〜1.0 | 0.0 | 音程切替タイミング |
| arpeggio.limit | integer | 0 | 切替回数の上限（0 = 無制限） |

### 4-6. フィルタ (Filter)

| パラメータ | 範囲 | デフォルト | 効果 |
|---|---|---|---|
| filter.cutoff | 0.0〜1.0 | 1.0 | ローパスカットオフ（1.0 = 全開） |
| filter.cutoff_sweep | -1.0〜1.0 | 0.0 | LPF カットオフの時間変化 |
| filter.resonance | 0.0〜1.0 | 0.0 | 共鳴（1.0 に近いと自己発振） |
| filter.highpass_cutoff | 0.0〜1.0 | 0.0 | ハイパスカットオフ |
| filter.highpass_sweep | -1.0〜1.0 | 0.0 | HPF カットオフの時間変化 |

### 4-7. フェイザー (Phaser)

| パラメータ | 範囲 | デフォルト | 効果 |
|---|---|---|---|
| phaser.offset | -1.0〜1.0 | 0.0 | フェイザーオフセット |
| phaser.sweep | -1.0〜1.0 | 0.0 | フェイザースイープ速度 |

### 4-8. その他

| パラメータ | 範囲 | デフォルト | 効果 |
|---|---|---|---|
| retrigger.repeat_speed | 0.0〜1.0 | 0.0 | リピート速度（0 = なし） |
| distortion.gain | 0.0〜1.0 | 0.0 | 歪みゲイン |
| distortion.compress_ratio | 0.0〜1.0 | 0.5 | コンプレッション比率 |
| bitcrusher.bit_depth | 1〜16 | 16 | ビット深度（16 = 劣化なし） |
| bitcrusher.sample_rate_reduction | 0〜N | 0 | サンプルレート間引き量（0 = なし） |

---

## 5. 生成エンジン 処理フロー

入力: ValidatedParams
出力: f32 PCM サンプル列（-1.0〜1.0）

### 処理ステップ

```
Step 1: エンベロープ長計算
  total_samples = (attack_time + sustain_time + decay_time) × sample_rate

Step 2: 合成ステート初期化（SynthState）
  fperiod, fslide, fphase, env_vol,
  fltp, fltdp, fltphp, iphase, ipp, ...

Step 3: サンプルループ（i = 0 .. total_samples）

  3a. リピートカウンタ確認
      repeat_time が来たら SynthState を部分リセット（エンベロープは維持）

  3b. アルペジオ更新
      arp_time が来たら fperiod *= arp_multiplier

  3c. 周波数スライド適用
      fslide += fdslide
      fperiod *= (1.0 - fslide)
      fperiod をクランプ（freq_limit 以下にならないよう）

  3d. ビブラート計算
      vib_phase += vib_speed
      rfperiod = fperiod × (1.0 + sin(vib_phase) × vib_strength)

  3e. 波形生成
      sample = oscillator(phase, rfperiod, wave_type, duty_cycle)
      phase += 1.0 / rfperiod

  3f. ローパスフィルタ適用
      lpf_freq += lpf_freq_sweep
      fltp += (sample - fltp) × lpf_freq
      fltdp += (fltp - fltdp) × resonance  ← Bfxr 独自係数
      sample = fltdp

  3g. ハイパスフィルタ適用
      hpf_freq += hpf_freq_sweep
      fltphp += fltp - fltphp
      sample -= fltphp × hpf_freq

  3h. フェイザー適用
      phaser_buf[ipp] = sample
      sample += phaser_buf[(ipp - iphase + 1024) & 1023]
      ipp = (ipp + 1) & 1023

  3i. エンベロープ乗算
      env_vol = envelope(i, attack_samples, sustain_samples, decay_samples, punch)
      sample *= env_vol

  3j. ゲイン・歪み適用
      sample *= volume * (1.0 + gain)
      sample = clamp(sample, -1.0, 1.0)

  3k. ビットクラッシャー適用
      （bit_depth < 16 の場合のみ）
      sample = round(sample × 2^bit_depth) / 2^bit_depth

Step 4: 出力正規化
  全サンプルの最大絶対値で割る（クリッピングを防ぐ）
```

### SynthState 構造体（Rust 参考）

```rust
struct SynthState {
    // 周波数
    fperiod: f64,
    fslide: f64,
    fdslide: f64,
    fphase: f64,
    // ビブラート
    vib_phase: f64,
    // エンベロープ
    env_stage: i32,
    env_time: i32,
    env_vol: f64,
    // フィルタ
    fltp: f64,
    fltdp: f64,
    fltphp: f64,
    // フェイザー
    phaser_buf: [f32; 1024],
    ipp: usize,
    iphase: i32,
    // リピート
    rep_time: i32,
    rep_limit: i32,
    // アルペジオ
    arp_time: i32,
    arp_limit: i32,
    arp_mod: f64,
}
```

---

## 6. パラメータ生成ツールの役割

### 責務分離の原則

```
preset  = 固定値の名前付きコレクション（変更なし、読み取り専用）
random  = カテゴリ別アルゴリズム生成（再現性のためシード管理）
mutate  = 既存パラメータへの確率的変異（変異強度を外部制御）
```

この 3 つは独立して呼び出せる。組み合わせは呼び出し元が制御する。

### params/preset

責務:
- 組み込みプリセット 16 種（coin, laser, explosion, powerup, hit, jump, blip, ...）を定義
- 外部 JSON ファイルからのカスタムプリセット読み込み
- カテゴリタグによる検索・一覧

責務外:
- プリセットの変更・上書き保存
- ランダム要素の追加

### params/random

責務:
- カテゴリ（pickup / laser / explosion / powerup / hit / jump / blip）別の生成アルゴリズム
- シード値 → 決定論的乱数列 → パラメータ生成（同シードなら同じ出力）
- 「完全ランダム」モード（全パラメータを独立サンプリング）

責務外:
- 生成後のパラメータの評価・選別
- WAV ファイルの出力

### params/mutate

責務:
- ベースパラメータ + 変異強度 (0.0〜1.0) → 新パラメータ
- 各パラメータに変異感度（mutability weight）を個別設定可能
- 変異分布の選択（正規分布 or 一様分布）
- ロックリスト（変異させないパラメータの指定）

責務外:
- ベースパラメータの生成

変異強度の意味:
- 0.0 → 入力と同一
- 0.3 → 小変化（微調整）
- 0.7 → 大変化（別物に近い）
- 1.0 → ほぼランダム

---

## 7. MVP 実装順序

### Phase 1: コア基盤（Week 1-2）

目標: `sfx generate coin.json -o coin.wav` が動く状態

1. パラメータスキーマ定義（schemas/sfx-params.v1.json）
2. params/validator（JSON 読み込み・デフォルト補完・値域チェック）
3. core/oscillator（square / sine / noise の 3 波形のみ）
4. core/envelope（attack / sustain / decay の基本形）
5. core/synthesizer（サンプルループの結合）
6. output/wav（44100Hz 16bit モノラル WAV エンコード）
7. cli/main（最小限の引数解析）

### Phase 2: 再現性向上（Week 3-4）

目標: Bfxr のプリセット音と聴き比べて「似ている」と言える状態

8. 全波形追加（sawtooth / triangle / breaker）
9. core/filter（LPF / HPF、Bfxr 係数に準拠）
10. 周波数スライド・デルタスライド
11. ビブラート
12. リピート（retrigger / repeat_speed）

### Phase 3: FX 追加（Week 5-6）

目標: Bfxr の全エフェクトが動く状態

13. core/arpeggio
14. core/phaser
15. core/bitcrusher
16. core/distortion（歪み・コンプレッサー）

### Phase 4: ツール整備（Week 7-8）

目標: preset / random / mutate が CLI から使える状態

17. params/preset（16 プリセット）
18. params/random（カテゴリ別ジェネレーター）
19. params/mutate
20. CLI の利便性向上（--play / --list-presets / --random / --mutate）

---

## 8. 技術選定

### 推奨構成: Rust（コア）+ TypeScript（ツール）

| 層 | 技術 | 理由 |
|---|---|---|
| コアエンジン | Rust | DSP 精度・速度・WebAssembly 化が容易 |
| WAV エンコーダ | hound crate | 実績ある Rust WAV ライブラリ |
| JSON 読み込み | serde_json crate | Rust 標準的な選択 |
| CLI | clap crate（derive feature） | 型安全な CLI 引数解析 |
| パラメータツール | TypeScript + Node.js | JSON スキーマ親和性・Web 移植が楽 |
| スキーマバリデーション | ajv（TypeScript 側） | JSON Schema Draft-07 対応 |
| テスト（Rust） | cargo test + approx crate | 浮動小数点の許容誤差比較 |
| テスト（TS） | Jest + ts-jest | TypeScript ネイティブ対応 |

### WebAssembly 化の見通し

```
Rust core
  → wasm-pack でビルド
  → .wasm + JavaScript グルーコード
  → ブラウザ上で WebAudio API と組み合わせて動作
```

コアを Rust で書く最大の理由は、将来 Web 化する際にコアを書き直さなくていいこと。

### TypeScript オールインの代替案

Rust 未経験の場合は TypeScript に一本化しても成立する。
効果音生成程度の速度要件は Node.js で十分に満たせる。
Web 化時も同じコードがブラウザで動く。
Rust 化は後から段階的に行える。

### 選択基準

- Rust に慣れている → 推奨構成
- TypeScript が得意で早く動かしたい → TypeScript オールイン
- どちらも初めて → TypeScript から始めて後で Rust 化

---

## 9. ディレクトリ構成

```
AiSound/                              # リポジトリルート（F:\App\Ohlo\AiSound）
│
├── Cargo.toml                        # Rust ワークスペース定義
│
├── core/                             # Rust コアエンジン
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                    # 公開 API（外部から見えるインターフェース）
│       ├── params.rs                 # パラメータ型定義（SfxParams 構造体）
│       ├── synthesizer.rs            # メインループ（SynthState + サンプル生成）
│       ├── oscillator.rs             # 波形生成
│       ├── envelope.rs               # エンベロープ計算
│       ├── filter.rs                 # LPF / HPF（Bfxr 係数）
│       ├── phaser.rs                 # フェイザー
│       ├── arpeggio.rs               # アルペジオ
│       └── bitcrusher.rs             # ビットクラッシャー
│
├── core/tests/
│   ├── integration_test.rs           # スナップショットテスト
│   └── fixtures/
│       ├── params/                   # テスト用 params.json 群
│       └── reference/               # Bfxr で生成した参照 WAV ファイル
│
├── cli/                              # CLI エントリーポイント
│   ├── Cargo.toml                    # core を依存に持つ
│   └── src/
│       └── main.rs
│
├── tools/                            # TypeScript ツール群
│   ├── package.json
│   ├── tsconfig.json
│   └── src/
│       ├── schema/
│       │   ├── params.schema.json    # JSON スキーマ（バリデーション用）
│       │   └── validator.ts          # ajv を使ったバリデーター
│       ├── preset/
│       │   ├── presets.ts            # プリセット読み込み・検索
│       │   └── builtin/              # coin.json / laser.json / ...
│       ├── random/
│       │   ├── generator.ts          # ランダム生成メイン
│       │   └── categories/           # pickup.ts / explosion.ts / ...
│       └── mutate/
│           └── mutator.ts            # 変異処理
│
├── tools/tests/                      # TypeScript テスト
│
├── schemas/
│   └── sfx-params.v1.json            # 共有スキーマ（Rust/TS 両方が参照）
│
├── presets/
│   ├── builtin/                      # 組み込みプリセット JSON
│   │   ├── coin.json
│   │   ├── laser.json
│   │   ├── explosion.json
│   │   └── ...（計 16 種）
│   └── user/                         # ユーザー追加プリセット
│
├── examples/
│   ├── coin.json                     # サンプルパラメータ（手書き）
│   └── explosion.json
│
├── docs/
│   ├── DESIGN.md                     # この設計書（本ファイル）
│   ├── architecture.md               # アーキテクチャ概要（設計書 Section 1-2 の要約）
│   ├── params-reference.md           # パラメータ全仕様（人間が読む用）
│   └── bfxr-mapping.md              # Bfxr AS3 変数名との対応表・変換式
│
├── .github/
│   └── workflows/
│       └── ci.yml                    # GitHub Actions CI
│
├── .gitignore
└── README.md
```

---

## 10. API 案

### 10-1. コアエンジン（Rust lib.rs）

```rust
/// パラメータ JSON バイト列から PCM サンプル列を生成
pub fn generate(params_json: &[u8]) -> Result<Vec<f32>, SfxError>

/// パラメータ構造体から直接生成（内部・Wasm 向け）
pub fn generate_from_params(params: &SfxParams) -> Result<Vec<f32>, SfxError>

/// PCM サンプル列を WAV バイト列に変換
pub fn encode_wav(samples: &[f32], sample_rate: u32, bit_depth: u8) -> Result<Vec<u8>, SfxError>

/// ショートカット: JSON → WAV バイト列（CLI / Wasm 向けのメイン API）
pub fn generate_wav(params_json: &[u8]) -> Result<Vec<u8>, SfxError>
```

設計の要点:
- 入力・出力は全てバイト列（ファイルシステム非依存）
- Wasm では同じ関数がそのままブラウザで動く
- SfxError は詳細なエラー種別を持つ（validation / synthesis / encode）

### 10-2. CLI コマンド

```sh
# 基本生成
sfx generate <params.json> [-o output.wav] [--play]

# プリセットから生成
sfx preset <name> [-o output.wav] [--play]
sfx preset --list [--category pickup]

# ランダム生成（params.json を出力）
sfx random [--category pickup] [--seed 42] [-o params.json]
sfx random [--category pickup] [--seed 42] --play

# 変異
sfx mutate <params.json> [--strength 0.3] [-o mutated.json] [--play]

# バリデーション
sfx validate <params.json>

# バッチ生成（ディレクトリ内の全 params.json を処理）
sfx batch <params_dir/> -o <output_dir/>
```

### 10-3. TypeScript ツール API

```typescript
// validator.ts
export function validate(json: unknown): SfxParams   // 失敗時は throw
export function applyDefaults(partial: Partial<SfxParams>): SfxParams

// preset/presets.ts
export function loadPreset(name: string): SfxParams
export function listPresets(category?: SfxCategory): PresetMeta[]
export function loadPresetFromFile(path: string): SfxParams

// random/generator.ts
export function generateRandom(options: {
  category: SfxCategory
  seed?: number
}): SfxParams

export function generateFullRandom(seed?: number): SfxParams

// mutate/mutator.ts
export function mutate(params: SfxParams, options: {
  strength: number          // 0.0 〜 1.0
  lock?: string[]           // 変異させないフィールドパス（例: "wave.type"）
  distribution?: 'normal' | 'uniform'
}): SfxParams
```

---

## 11. テスト戦略

### テストレベルと目的

**Unit Test（各モジュール単体）**

対象: oscillator / envelope / filter / validator

- oscillator: 各波形の値域（-1.0〜1.0）・周期・duty_cycle の反映を数値検証
- envelope: attack / sustain / decay 各フェーズの切替タイミングを検証
- filter: インパルス応答でカットオフ周波数の実効値を検証
- validator: 境界値・型違反・欠損フィールド・バージョン違いを網羅

**Integration Test（コア結合）**

対象: core/synthesizer 全体

- 既知パラメータ → 既知 WAV 出力の一致（スナップショットテスト）
- Bfxr オリジナル WAV との相関係数 > 0.95 を確認
- サイレンス（全サンプルが 0）が出力されていないことの確認
- クリッピング（絶対値 > 1.0 のサンプル）が出力されていないことの確認

**Regression Test（Bfxr 再現性）**

対象: 参照 WAV ファイルとの比較

- fixtures/reference/ に Bfxr で生成した WAV を保存しておく
- CI で同パラメータを使って自前エンジンで生成し相関係数を測定
- 閾値を下回ったら CI 失敗にする

**Property Test（ランダム生成の健全性）**

対象: params/random + core/synthesizer

- 任意シードの random 生成 → synthesizer でパニックしない
- 出力サンプル数がエンベロープ長から計算できる範囲内に収まる
- 出力値が -1.0〜1.0 に収まる

### テスト用ヘルパー

```rust
// tests/helpers.rs
fn load_params(fixture: &str) -> SfxParams  // fixtures/params/ からロード
fn load_reference_wav(name: &str) -> Vec<f32>  // fixtures/reference/ からロード
fn correlation(a: &[f32], b: &[f32]) -> f32  // ピアソン相関係数
fn rms(samples: &[f32]) -> f32  // RMS 値
```

---

## 12. 注意点

### 12-1. Bfxr 再現性

Bfxr のソースコードは公開されている（AS3 実装）。
数式レベルの一致を目指せるが、以下の点に注意：

- 周波数変換の二乗カーブと、フィルタ係数の計算式は必ず元ソースと照合すること
- フィルタは Bfxr 独自の簡易実装。一般的な Biquad とは係数が異なる
- 「Bfxr 完全互換」か「標準的な DSP」かを明示的に決めておく（混在は禁止）
- フェイザーの実装は複雑。MVP では後回しにしても品質への影響は小さい

### 12-2. パラメータファイルの長期運用

- version フィールドとマイグレーション関数を最初から用意する
  - 後から追加すると既存ファイル群が全て壊れる
- フィールド名は意味ベースで命名する（Bfxr の AS3 変数名は一貫性が低い）
- meta.seed は記録するが、乱数ライブラリのバージョン変更で再現できなくなる場合がある
  - 完全な再現性は保証しないと README に明記する
- パラメータファイルを長期保存する場合は Git で管理する（バイナリ WAV でなく JSON で保管）

### 12-3. Web 化に向けた注意

- Rust の f64 演算はターゲット（x86/Wasm）で結果が微妙に異なる場合がある
  - テストの数値比較は厳密一致でなく許容誤差（例: 1e-6）を設ける
- Wasm ではファイルシステムが使えない
  - コア API を「バイト列入出力のみ」に設計する（Section 10-1 で対応済み）
  - lib.rs に fs::read / fs::write を書かない
- Web 化時は wasm-pack を使う（wasm-bindgen の手動設定は不要）

### 12-4. 実装順序の罠

以下の順序ミスに注意：

- フィルタを Bfxr 係数で実装する前に「だいたい合ってる」で進めると後で全部作り直しになる
- 周波数変換の二乗カーブを線形で実装すると音域が全く違うものになる
- エンベロープ長の計算式を間違えると他のテストが全て崩れる
- これらは Day 1 に bfxr-mapping.md を書きながら確認すること

---

## 13. GitHub 運用方針

### リポジトリ設定

- リポジトリ名: AiSound
- ローカルパス: F:\App\Ohlo\AiSound
- デフォルトブランチ: main
- 可視性: Public（将来の OSS 化を視野に入れる）

### .gitignore

```
# Rust
target/
Cargo.lock（ライブラリクレートの場合）

# Node.js
node_modules/
dist/
*.js.map

# 生成物（バイナリは管理しない）
*.wav
*.mp3

# OS
.DS_Store
Thumbs.db
```

### ブランチ戦略（GitHub Flow）

```
main                  常にビルド・テスト通過。直接 push 禁止。
feat/oscillator       機能追加
feat/envelope
feat/filter
fix/freq-conversion   バグ修正
docs/params-ref       ドキュメント更新
```

- main への変更は必ず PR 経由
- PR マージ前に CI が通っていることを確認

### コミットメッセージ規則（Conventional Commits）

```
feat: oscillator に sawtooth 波形を追加
fix: 周波数変換式の二乗カーブが線形になっていたのを修正
docs: bfxr-mapping.md にフィルタ係数の対応表を追加
test: envelope のスナップショットテストを追加
chore: GitHub Actions CI を設定
refactor: SynthState を構造体に分離
```

### GitHub Actions CI

```yaml
# .github/workflows/ci.yml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-core:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
      - run: cargo clippy -- -D warnings

  test-tools:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: cd tools && npm ci && npm test
```

### Issues / マイルストーン

ラベル:
- core（コアエンジン実装）
- tools（パラメータツール）
- docs（ドキュメント）
- bug
- enhancement

マイルストーン（Section 7 の Phase に対応）:
- Phase 1: コア基盤（Week 1-2）
- Phase 2: 再現性向上（Week 3-4）
- Phase 3: FX 追加（Week 5-6）
- Phase 4: ツール整備（Week 7-8）

---

## 14. 最初の 1 週間 作業リスト

**重要: Day 1 の Bfxr ソース読解を絶対に省かないこと。**
フィルタと周波数変換の独自実装を理解せずに実装を始めると、後で全部作り直しになる。

### Day 1（優先度: 最高）

作業:
1. GitHub リポジトリ作成（AiSound）、ローカルと紐付け
2. ディレクトリ骨格作成、.gitignore / README.md を追加
3. Bfxr AS3 ソースを読む（URL: https://github.com/increpare/bfxr）
   - 重点: Synth.as の generateBlip() 関数
   - 周波数変換式を確認して docs/bfxr-mapping.md に記録
   - フィルタ係数の計算式を確認して記録
4. schemas/sfx-params.v1.json を作成・コミット
5. examples/coin.json を手書きで作成

成果物: リポジトリ / docs/bfxr-mapping.md / schemas/ / examples/coin.json

### Day 2-3（優先度: 最高）

作業:
1. Rust ワークスペース初期化（workspace Cargo.toml）
2. core/src/params.rs（SfxParams 構造体 + serde）
3. core/src/oscillator.rs（square 波形のみ）
4. core/src/envelope.rs（sustain + decay のみ）
5. core/src/synthesizer.rs（サンプルループの骨格）
6. cargo build が通ることを確認

成果物: core/ の骨格実装

### Day 4（優先度: 最高）

作業:
1. output/wav（hound crate で 44100Hz / 16bit / モノラル WAV 出力）
2. cli/src/main.rs（`sfx generate <params.json> -o <output.wav>` の最小実装）
3. examples/coin.json を入力として coin.wav が生成されることを確認

成果物: 「音が鳴る」状態

### Day 5（優先度: 最高）

作業:
1. Bfxr の Web 版（https://www.bfxr.net/）で coin プリセットの WAV を生成・保存
2. 自作エンジンの出力と耳で比較
3. 明らかな差異をメモ → docs/bfxr-mapping.md の TODO 欄に追記
4. 差異の大きい順に Week 2 の実装優先度を決める

成果物: fixtures/reference/coin.wav / 差異リスト

### Day 6（優先度: 中）

作業:
1. cargo test の骨格（fixtures/coin.json → coin.wav のスナップショットテスト 1 本）
2. GitHub Actions ci.yml 作成・push して CI が通ることを確認

成果物: .github/workflows/ci.yml / 最初のテスト

### Day 7（優先度: 中）

作業:
1. docs/architecture.md（設計書 Section 1-2 の要約）
2. README.md にビルド手順・動かし方を記載
3. docs/params-reference.md の着手（Section 4 の表をコピーして整形）
4. GitHub Issues に Phase 1 のタスクを登録、マイルストーンを設定

成果物: docs/ の整備 / GitHub Issues

### Week 2 以降（参考）

- sine / noise 波形追加
- LPF フィルタ実装（Bfxr 係数に照合しながら）
- 周波数スライド・デルタスライド実装
- TypeScript ツール層の骨格（package.json / tsconfig.json）
- params/random の pickup カテゴリのみ実装

---

## 付録: Claude Code への初期化指示

以下を Claude Code に渡してプロジェクトを初期化する。

```
作業ディレクトリ: F:\App\Ohlo\AiSound
設計書: docs/DESIGN.md（本ファイル）を参照しながら進めること

## タスク: プロジェクト骨格の作成

### 1. ディレクトリ構成を作る（設計書 Section 9 を参照）

以下のディレクトリを作成する:
  core/src/
  core/tests/fixtures/params/
  core/tests/fixtures/reference/
  cli/src/
  tools/src/schema/
  tools/src/preset/builtin/
  tools/src/random/categories/
  tools/src/mutate/
  tools/tests/
  schemas/
  presets/builtin/
  presets/user/
  examples/
  docs/
  .github/workflows/

### 2. 設定ファイルを作成する

.gitignore（設計書 Section 13 を参照）
README.md（プロジェクト概要・ビルド手順は TODO で残す）

workspace の Cargo.toml:
  [workspace]
  members = ["core", "cli"]
  resolver = "2"

core/Cargo.toml:
  [package]
  name = "sfx-core"
  version = "0.1.0"
  edition = "2021"
  [dependencies]
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  [dev-dependencies]
  approx = "0.5"

cli/Cargo.toml:
  [package]
  name = "sfx-cli"
  version = "0.1.0"
  edition = "2021"
  [dependencies]
  sfx-core = { path = "../core" }
  clap = { version = "4", features = ["derive"] }

core/src/lib.rs に TODO コメントのみ記載

cli/src/main.rs に骨格のみ:
  fn main() { println!("AiSound CLI – coming soon"); }

cargo build --workspace が通ることを確認する

tools/ で npm init して TypeScript / Jest / ajv をインストール

### 3. スキーマとサンプルファイルを作成する

schemas/sfx-params.v1.json（設計書 Section 3-1 の JSON をそのまま使う）
examples/coin.json（設計書 Section 14 Day 1 の仕様で作成）

### 4. ドキュメントを作成する

docs/DESIGN.md（このファイル自体をコピー）
docs/bfxr-mapping.md:
  - Bfxr ソース URL: https://github.com/increpare/bfxr
  - 周波数変換式（設計書 Section 4-1）
  - 各パラメータとの対応表（TODO: Bfxr AS3 変数名は後で記入）

### 5. GitHub Actions を設定する

.github/workflows/ci.yml（設計書 Section 13 の YAML をそのまま使う）

### 6. git init して最初のコミットを作る

コミットメッセージ: "chore: initial project structure"

GitHub に AiSound リポジトリを作成し、origin に設定して push する

### 7. GitHub Issues を登録する（設計書 Section 14 を参照）

Phase 1 マイルストーンを作成して以下の Issues を登録:
  - [Phase 1] oscillator 実装（square/sine/noise）ラベル: core
  - [Phase 1] envelope 実装（ADSR）ラベル: core
  - [Phase 1] synthesizer 実装（メインループ）ラベル: core
  - [Phase 1] WAV エンコーダ実装 ラベル: core
  - [Phase 1] CLI 最小版実装 ラベル: core
  - [Phase 1] bfxr-mapping.md 完成（Bfxr ソース読解）ラベル: docs
```

---

*設計書 v1.0 終わり*
