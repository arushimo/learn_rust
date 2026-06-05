# EV Charging API

This project is an EV (Electric Vehicle) Charging API implemented in Rust, utilizing gRPC and a PostgreSQL database.

## ディレクトリ構成

最新のファイル構成は以下の通りです。

```text
ev_charging_api/
├── build.rs                   # gRPC (Tonic) などのビルドスクリプト
├── Cargo.lock
├── Cargo.toml                 # Rustのパッケージ・依存関係定義
├── compose.yaml               # Docker Composeの設定 (PostgreSQL等)
├── Dockerfile                 # アプリケーションのDockerイメージ定義
├── init.sql                   # データベースの初期化スクリプト
├── Makefile                   # ビルド、実行、テスト用のタスクランナー
├── README.md                  # 本ファイル
├── proto/                     # Protocol Buffersの定義ファイル
│   └── charging.proto         # gRPCのインターフェース定義
├── src/                       # Rustソースコード
│   ├── main.rs                # アプリケーションのエントリーポイント
│   ├── domain/                # ドメイン層 (ビジネスロジックのコア)
│   │   ├── charge_session.rs  # 充電セッションのエンティティ
│   │   ├── kwh.rs             # 電力(kWh)のバリューオブジェクト
│   │   ├── mod.rs
│   │   └── repository.rs      # データアクセスのためのインターフェース(トレイト)
│   ├── infrastructure/        # インフラストラクチャ層 (外部システムとの連携)
│   │   ├── mod.rs
│   │   └── postgres_repository.rs # PostgreSQLへのアクセス実装
│   ├── presentation/          # プレゼンテーション層 (APIエンドポイント)
│   │   ├── grpc_handler.rs    # gRPCリクエストのハンドラ実装
│   │   └── mod.rs
│   └── usecase/               # ユースケース層 (アプリケーションロジック)
│       ├── charge_session_usecase.rs # 充電セッションの処理フロー実装
│       └── mod.rs
├── target/                    # ビルド成果物 (自動生成)
└── tests/                     # 統合テスト
    └── api_test.rs            # gRPC APIの結合テストコード
```

## 設計方針

本プロジェクトは、**クリーンアーキテクチャ（Clean Architecture） / オニオンアーキテクチャ（Onion Architecture）** の考え方をベースに、関心の分離とテスト容易性を重視した層（レイヤー）ごとの設計を採用しています。

### レイヤー構成

1. **ドメイン層 (`src/domain/`)**
   - **責務:** システムのコアとなるビジネスルール、エンティティ（例: `charge_session.rs`）、バリューオブジェクト（例: `kwh.rs`）を定義します。
   - **特徴:** 外部のフレームワークやデータベースなどの詳細技術（DBやgRPCなど）には一切依存しません。データアクセスのための抽象（`repository.rs` に定義されるトレイト）もここで提供します。

2. **ユースケース層 (`src/usecase/`)**
   - **責務:** アプリケーション固有のビジネスロジック（ユースケース）のフローを制御します（例: `charge_session_usecase.rs`）。
   - **特徴:** プレゼンテーション層からの入力を受け取り、ドメイン層のエンティティやリポジトリ（トレイト）をオーケストレーションして、システムが提供すべき機能を実現します。インフラストラクチャの具体的な実装には依存しません。

3. **インフラストラクチャ層 (`src/infrastructure/`)**
   - **責務:** 外部システム（データベース等）との通信などの技術的詳細を担当します。
   - **特徴:** ドメイン層で定義されたリポジトリのインターフェース（トレイト）を実装します（例: `postgres_repository.rs` での PostgreSQL 操作）。

4. **プレゼンテーション層 (`src/presentation/`)**
   - **責務:** クライアントからの外部リクエストを受け付け、ユースケース層の処理を呼び出し、レスポンスを返却します。
   - **特徴:** 本プロジェクトでは gRPC を採用しており、`grpc_handler.rs` にて `proto` で定義したインターフェースに合わせたエンドポイントを実装しています。

### 技術スタック
- **言語:** Rust
- **API 通信:** gRPC (Tonic, Prost)
- **データベース:** PostgreSQL
- **インフラ/コンテナ:** Docker, Docker Compose
