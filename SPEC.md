# Runeforge

---

## 0. ゴール

CLI で **Blueprint**（要件記述）を読み取り、→ **最適スタック**を JSON で返す。  
生成した `plan.json` は RuneWeave 以降にそのまま渡せる。

---

## 1. 入出力

|種別|形式|スキーマ|
|---|---|---|
|入力|`blueprint.yaml` / `blueprint.json`|`schemas/blueprint.schema.json`|
|出力|`plan.json`|`schemas/stack.schema.json`|

### 1.1 CLI

```bash
runeforge plan \
  -f blueprint.yaml   # 必須
  --seed 42           # 同一入力+seed でハッシュ不変
  --out plan.json     # 省略時 stdout
  --strict            # schema NG で exit≠0
```

|Exit|意味|
|---|---|
|0|正常|
|1|入力スキーマ不一致|
|2|出力スキーマ不一致|
|3|条件に合うスタックなし|

---

## 2. スキーマ

### 2.1 `schemas/blueprint.schema.json`（抜粋）

```jsonc
{
  "type": "object",
  "required": ["project_name", "goals", "constraints", "traffic_profile"],
  "properties": {
    "project_name": { "type": "string", "minLength": 1 },
    "goals": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
    "constraints": {
      "type": "object",
      "properties": {
        "monthly_cost_usd_max": { "type": "number", "minimum": 0 },
        "persistence": { "enum": ["kv","sql","both"] },
        "region_allow": { "type": "array", "items": { "type": "string" } },
        "compliance": {
          "type": "array",
          "items": { "enum": ["audit-log","sbom","pci","sox","hipaa"] }
        }
      },
      "additionalProperties": false
    },
    "traffic_profile": {
      "type": "object",
      "required": ["rps_peak","global","latency_sensitive"],
      "properties": {
        "rps_peak": { "type": "number", "minimum": 0 },
        "global": { "type": "boolean" },
        "latency_sensitive": { "type": "boolean" }
      }
    },
    "prefs": {
      "type": "object",
      "properties": {
        "frontend": { "type": "array", "items": { "type": "string" } },
        "backend":  { "type": "array", "items": { "type": "string" } },
        "database": { "type": "array", "items": { "type": "string" } },
        "ai":       { "type": "array", "items": { "type": "string" } }
      },
      "additionalProperties": false
    },
    "single_language_mode": { "enum": ["rust","go","ts",null] }
  },
  "additionalProperties": false
}
```

### 2.2 `schemas/stack.schema.json`（抜粋）

```jsonc
{
  "type": "object",
  "required": ["decisions","stack","estimated","meta"],
  "properties": {
    "decisions": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["topic","choice","reasons","alternatives","score"],
        "properties": {
          "topic": { "type": "string" },
          "choice": { "type": "string" },
          "reasons": { "type": "array", "items": { "type": "string" } },
          "alternatives": { "type": "array", "items": { "type": "string" } },
          "score": { "type": "number" }
        }
      }
    },
    "stack": {
      "type": "object",
      "required": ["language","frontend","backend","database","cache","queue","ai","infra","ci_cd"],
      "properties": {
        "language": { "type": "string" },
        "frontend": { "type": "string" },
        "backend":  { "type": "string" },
        "database": { "type": "string" },
        "cache":    { "type": "string" },
        "queue":    { "type": "string" },
        "ai":       { "type": "array", "items": { "type": "string" } },
        "infra":    { "type": "string" },
        "ci_cd":    { "type": "string" }
      }
    },
    "estimated": {
      "type": "object",
      "required": ["monthly_cost_usd"],
      "properties": { "monthly_cost_usd": { "type": "number", "minimum": 0 } }
    },
    "meta": {
      "type": "object",
      "required": ["seed","blueprint_hash","plan_hash"],
      "properties": {
        "seed": { "type": "integer" },
        "blueprint_hash": { "type": "string" },
        "plan_hash": { "type": "string" }
      }
    }
  }
}
```

---

## 3. 選定ロジック

1. **正規化** — `blueprint.(yaml|json)` を内部構造体へ。
    
2. **候補列挙** — `resources/rules.yaml` に候補と制約（対応リージョン、依存性、前提コスト等）を定義。
    
3. **スコアリング**
    
    ```text
    score = Σ weight_i × metric_i
    weights: quality 0.30 / slo 0.25 / cost 0.20 / security 0.15 / ops 0.10
    metrics 例:
      quality: コミュニティ成熟度・ドキュメント充実度
      slo: 低遅延対応・水平分散適性
      cost: ランニング+egress 推定
      security: SBOM/署名/ゼロトラスト適性
      ops: CI/CD 一体化容易性・IaC との親和性
    ```
    
4. **フィルタ** — コスト上限・リージョン許可・コンプライアンスで除外。
    
5. **決定性** — `--seed` で RNG を固定。同点は `tie_breaker(topic, seed)` で安定化。
    
6. **根拠生成** — 1 位と次善案を `decisions[]` に格納（`reasons[]` 付き）。
    
7. **スキーマ検証** — `stack.schema.json` に適合しなければ exit=2。
    

---

## 4. 受け入れ基準

|#|条件|
|---|---|
|1|入力スキーマ NG → exit 1|
|2|出力スキーマ NG → exit 2|
|3|同入力+seed で `meta.plan_hash` 不変（SHA-256/hex）|
|4|`decisions[*]` に理由・代替・スコアが入る|
|5|付属テストケース（baseline / latency / compliance）3 つがすべて緑|

---

## 5. 実装レイアウト

```
/src
  main.rs        # clap CLI（サブコマンド plan）
  selector.rs    # 候補列挙 + スコアリング + tie-break
  schema.rs      # schemars で I/O 検証
  util.rs        # hash（sha256+hex）・決定性 RNG（seed）
/resources
  rules.yaml     # 候補・重み・前提コスト・適用条件
/schemas
  blueprint.schema.json
  stack.schema.json
/examples
  baseline.yaml
  latency.yaml
  compliance.yaml
```

### 依存（固定）

|crate|ver|用途|
|---|---|---|
|`clap`|4.5|CLI パーサ（derive） ([Docs.rs](https://docs.rs/crate/clap/latest?utm_source=chatgpt.com "clap 4.5.42 - Docs.rs"))|
|`serde` / `serde_json` / `serde_yaml`|1 / 1 / 0.9系|JSON/YAML I/O（derive） ([Docs.rs](https://docs.rs/crate/serde/latest?utm_source=chatgpt.com "serde 1.0.219 - Docs.rs"))|
|`schemars`|0.8|JSON Schema 生成・検証 ([Docs.rs](https://docs.rs/schemars/latest/schemars/?utm_source=chatgpt.com "schemars - Rust - Docs.rs"), [Graham’s Cool Site](https://graham.cool/schemars/v0/?utm_source=chatgpt.com "Overview \| Schemars"))|
|`rand`|0.8|決定性 RNG（seed 固定） ([Docs.rs](https://docs.rs/crate/rand/0.8.0?utm_source=chatgpt.com "rand 0.8.0 - Docs.rs"))|
|`sha2` + `hex`|0.10 / 0.4|ハッシュ（plan/blueprint） ([Docs.rs](https://docs.rs/crate/sha2/latest?utm_source=chatgpt.com "sha2 0.10.9 - Docs.rs"))|

`Cargo.toml` の `rust-version = "1.80"` を明記（MSRV 固定）。Rust 1.80+ でのビルドを前提とする。 ([GitHub](https://github.com/dtolnay/rust-toolchain/blob/master/action.yml?utm_source=chatgpt.com "rust-toolchain/action.yml at master · dtolnay/rust-toolchain · GitHub"))

---

## 6. `resources/rules.yaml`（構造例）

```yaml
version: 1
weights:
  quality: 0.30
  slo: 0.25
  cost: 0.20
  security: 0.15
  ops: 0.10

candidates:
  backend:
    - name: "Actix Web"
      requires: { language: "Rust" }
      metrics: { quality: 0.9, slo: 0.9, cost: 0.7, security: 0.8, ops: 0.8 }
      regions: ["*"]
      notes: ["成熟度と高スループット"]
    - name: "Axum"
      requires: { language: "Rust" }
      metrics: { quality: 0.85, slo: 0.85, cost: 0.7, security: 0.8, ops: 0.85 }
      regions: ["*"]

  frontend:
    - name: "SvelteKit"
      metrics: { quality: 0.85, slo: 0.8, cost: 0.8, security: 0.8, ops: 0.85 }
      regions: ["*"]

  database:
    - name: "PlanetScale"
      metrics: { quality: 0.8, slo: 0.85, cost: 0.75, security: 0.85, ops: 0.8 }
      regions: ["us","eu","apac"]

  cache:
    - name: "Cloudflare KV"
      metrics: { quality: 0.8, slo: 0.9, cost: 0.85, security: 0.85, ops: 0.9 }
      regions: ["global"]

  queue:
    - name: "NATS JetStream"
      metrics: { quality: 0.85, slo: 0.9, cost: 0.8, security: 0.85, ops: 0.85 }
      regions: ["*"]

  ai:
    - name: "RuneSage"
      metrics: { quality: 0.8, slo: 0.8, cost: 0.9, security: 0.85, ops: 0.8 }
      regions: ["*"]
    - name: "OpenAI GPT-4o"
      metrics: { quality: 0.95, slo: 0.9, cost: 0.7, security: 0.85, ops: 0.85 }
      regions: ["*"]

  infra:
    - name: "Terraform + Cloudflare Workers (wasm32-unknown-unknown)"
      metrics: { quality: 0.85, slo: 0.9, cost: 0.85, security: 0.85, ops: 0.9 }
      regions: ["global"]

  ci_cd:
    - name: "GitHub Actions"
      metrics: { quality: 0.9, slo: 0.9, cost: 0.85, security: 0.85, ops: 0.95 }
      regions: ["*"]
```

---

## 7. 出力サンプル（`plan.json` 抜粋）

```jsonc
{
  "decisions": [
    {
      "topic": "backend",
      "choice": "Actix Web",
      "reasons": ["高スループット/成熟度", "SLO重視のスコアが最上位"],
      "alternatives": ["Axum"],
      "score": 0.862
    }
  ],
  "stack": {
    "language":  "Rust",
    "frontend":  "SvelteKit",
    "backend":   "Actix Web",
    "database":  "PlanetScale",
    "cache":     "Cloudflare KV",
    "queue":     "NATS JetStream",
    "ai":        ["RuneSage","OpenAI GPT-4o"],
    "infra":     "Terraform + Cloudflare Workers (wasm32-unknown-unknown)",
    "ci_cd":     "GitHub Actions"
  },
  "estimated": { "monthly_cost_usd": 110 },
  "meta": {
    "seed": 42,
    "blueprint_hash": "sha256:...",
    "plan_hash": "sha256:..."
  }
}
```

---

## 8. テスト

- `cargo test` で以下を検証：
    
    - 入力スキーマ適合（NG で exit=1） — `schemars` による検証。 ([Docs.rs](https://docs.rs/schemars/latest/schemars/?utm_source=chatgpt.com "schemars - Rust - Docs.rs"))
        
    - 出力スキーマ適合（NG で exit=2）
        
    - 決定論的ハッシュ（同入力+seed → `meta.plan_hash` 不変）
        
    - 3 ケース（`examples/baseline.yaml` / `latency.yaml` / `compliance.yaml`）が緑
        

---

## 9. CI（最小）

- **GitHub Actions**：`dtolnay/rust-toolchain` でツールチェイン導入 → `Swatinem/rust-cache` → `cargo test --locked`。 ([GitHub](https://github.com/dtolnay/rust-toolchain?utm_source=chatgpt.com "GitHub - dtolnay/rust-toolchain: Concise GitHub Action for installing a ..."))
    
- 推奨 `permissions`：`contents: read`。
    
- `ubuntu-24.04` ランナー固定。
    
- 例：
    

```yaml
name: runeforge-ci
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable        # Rust導入
      - uses: Swatinem/rust-cache@v2               # キャッシュ
      - run: cargo test --locked
```

---

## 10. 実装開始（コマンド）

```bash
cargo new --bin runeforge && cd runeforge
# ├─ src/{main.rs,selector.rs,schema.rs,util.rs}
# ├─ resources/rules.yaml
# └─ schemas/{blueprint.schema.json,stack.schema.json}
```

— 依存追加：`clap = "4.5"`, `serde = { version = "1", features = ["derive"] }`,  
`serde_json = "1"`, `serde_yaml = "0.9"`, `schemars = "0.8"`,  
`rand = "0.8"`, `sha2 = "0.10"`, `hex = "0.4"`（MSRV 1.80 固定）。 ([Docs.rs](https://docs.rs/crate/clap/latest?utm_source=chatgpt.com "clap 4.5.42 - Docs.rs"))

---