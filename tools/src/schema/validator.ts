// TODO: JSON スキーマバリデーション実装
// - ajv を使って schemas/sfx-params.v1.json に対してバリデーション
// - デフォルト値の補完
// - 値域チェック
//
// 設計書 Section 10-3 参照:
//   export function validate(json: unknown): SfxParams
//   export function applyDefaults(partial: Partial<SfxParams>): SfxParams
