# Runeforge

## 0. ゴール

Runeforge は CLI ツールです。  
**Blueprint（要件記述）** を入力とし、要件に最適化された **技術スタック構成（plan.json）** を決定的に生成します。

- サービス単位で **言語・フレームワーク・ランタイム** を含む Polyglot 構成に対応。
    
- 生成物は **RuneWeave 以降の工程にそのまま渡せる**。
    
- **同じ入力 + seed** であれば必ず同じ結果を返す決定論的設計。
    

---

## 1. CLI 仕様

```bash
runeforge plan \
  -f blueprint.yaml   # 必須: Blueprint入力
  --seed 42           # 決定性のための乱数シード
  --out plan.json     # 出力先（省略時は stdout）
  --strict            # スキーマ検証NGなら exit≠0
  --beam 8            # サービス組合せ探索幅（既定=8）
```

### 終了コード

|Code|意味|
|---|---|
|0|正常終了|
|1|入力スキーマ不一致|
|2|出力スキーマ不一致|
|3|条件に合う候補が存在しない|

---

## 2. スキーマ

### 2.1 Blueprint (`schemas/blueprint.schema.json`)

Blueprint は要件定義です。必須項目は以下です。

- `project_name`: プロジェクト名
    
- `goals`: 達成したい目標リスト
    
- `constraints`: コスト上限・コンプライアンス・利用可能リージョンなど
    
- `traffic_profile`: RPSピークやグローバル展開要否など
    
- `prefs`: 開発者の嗜好（使いたい/避けたい技術スタック）
    
- `single_language_mode`: 全サービスを特定言語で統一したい場合に指定（例: `"rust"`, `"ts"`）
    

### 2.2 Stack (`schemas/stack.schema.json`)

Runeforge が返す plan.json の仕様です。

- `decisions[]`: 各決定に関する選択理由・代替案・スコア
    
- `stack`: 実際に選ばれたスタック構成
    
    - `language`: 単一言語モード互換用（Polyglot時は代表言語を記録）
        
    - `services[]`: 各サービスの言語・フレームワーク・ランタイム
        
    - その他、`frontend` / `backend` / `database` / `cache` / `queue` / `ai` / `infra` / `ci_cd`
        
- `estimated`: コスト推定など
    
- `meta`: seed やハッシュ情報
    

---

## 3. 選定ロジック

1. **正規化**  
    Blueprint を内部構造体に読み込む。
    
2. **候補列挙**  
    `resources/rules.yaml` に定義された「サービス種別 × 言語 × フレームワーク」候補を取得。
    
3. **スコアリング**
    
    ```
    score = Σ (weight_i × metric_i) - penalty
    ```
    
    - quality (0.30) / slo (0.25) / cost (0.20) / security (0.15) / ops (0.10)
        
    - penalty: 言語が増えるごとの複雑性、パッケージ管理の分散など
        
4. **フィルタ**  
    コスト上限、許可リージョン、コンプライアンス要件を満たさない候補を除外。
    
5. **決定性**  
    同点時は seed を使って安定的に tie-break。
    
6. **根拠生成**  
    採択理由、次善候補、スコアを `decisions[]` に保存。
    
7. **スキーマ検証**  
    出力が `stack.schema.json` に適合しなければ exit=2。
    

---

## 4. 出力サンプル

```jsonc
{
  "decisions": [
    {
      "topic": "api.framework",
      "choice": "Actix Web",
      "reasons": ["高スループット", "成熟度が高い"],
      "alternatives": ["Fastify"],
      "score": 0.87
    }
  ],
  "stack": {
    "language": "Rust",
    "services": [
      {
        "name": "api",
        "kind": "api",
        "language": "Rust",
        "framework": "Actix Web",
        "runtime": "rust@1.82",
        "build": "cargo",
        "tests": "cargo test"
      },
      {
        "name": "edge",
        "kind": "edge",
        "language": "TypeScript",
        "framework": "Cloudflare Workers",
        "runtime": "node@22",
        "build": "pnpm",
        "tests": "pnpm test"
      }
    ],
    "database": "PlanetScale",
    "cache": "Cloudflare KV",
    "queue": "NATS JetStream",
    "ai": ["OpenAI GPT-4o"],
    "infra": "Terraform + Cloudflare",
    "ci_cd": "GitHub Actions"
  },
  "estimated": {
    "monthly_cost_usd": 120,
    "egress_gb": 200,
    "notes": ["対象リージョン: us/eu/apac"]
  },
  "meta": {
    "seed": 42,
    "blueprint_hash": "sha256:...",
    "plan_hash": "sha256:..."
  }
}
```

---

## 5. 実装構成

```
/src
  main.rs        # CLI (clap)
  selector.rs    # 候補列挙・スコアリング・tie-break
  schema.rs      # 入出力スキーマ検証
  util.rs        # ハッシュ・決定性RNG
/resources
  rules.yaml     # 候補と評価指標
/schemas
  blueprint.schema.json
  stack.schema.json
/examples
  baseline.yaml
  latency.yaml
  compliance.yaml
```

### 主な依存クレート

- `clap` 4.5（CLI）
    
- `serde` / `serde_json` / `serde_yaml`（I/O）
    
- `schemars`（スキーマ検証）
    
- `rand` 0.8（決定性RNG）
    
- `sha2` + `hex`（ハッシュ）
    

MSRV = Rust 1.82 固定。

---

## 6. 受け入れ基準

- 入力スキーマ不一致 → exit 1
    
- 出力スキーマ不一致 → exit 2
    
- 同一入力 + seed → plan_hash が不変
    
- decisions[] に理由・代替・スコアが記録されている
    
- baseline / latency / compliance のテストケースが全て緑
    

---

## 7. CI 最小構成（例）

```yaml
name: runeforge-ci
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --locked
```

---

## まとめ

Runeforge は、Blueprint を基に **Polyglot 技術スタックを決定論的に導出するプランナー**です。

- サービスごとの言語・フレームワーク選定が可能。
    
- コスト、性能、コンプライアンス要件を満たした最適解を返す。
    
- RuneWeave 以降の工程とシームレスに連携できる。