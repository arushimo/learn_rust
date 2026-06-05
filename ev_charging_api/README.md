```
ev_charging_api/
├── build.rs               # gRPC (Tonic) などのビルドスクリプト
├── Cargo.lock
├── Cargo.toml             # Rustのパッケージ・依存関係定義
├── compose.yaml           # Docker Composeの設定 (DB等)
├── Dockerfile             # アプリケーションのDockerイメージ定義
├── init.sql               # データベースの初期化スクリプト
├── Makefile               # ビルドやテスト用のタスクランナー
├── README.md
├── proto/                 # Protocol Buffersの定義ファイル
│   └── charging.proto     # gRPCのインターフェース定義
├── src/                   # Rustのソースコード (オニオン/クリーンアーキテクチャ風の構成)
│   ├── main.rs            # エントリーポイント
│   ├── domain/            # ドメイン層 (エンティティやビジネスルール)
│   │   └── mod.rs
│   ├── infrastructure/    # インフラストラクチャ層 (DBアクセスなど)
│   │   ├── mod.rs
│   │   └── postgres_repository.rs # PostgreSQLへのアクセス実装
│   ├── presentation/      # プレゼンテーション層 (外部からのリクエスト受け付け)
│   │   ├── grpc_handler.rs        # gRPCのハンドラ実装
│   │   └── mod.rs
│   └── usecase/           # ユースケース層 (アプリケーションのビジネスロジック)
│       ├── charge_session_usecase.rs
│       └── mod.rs
├── target/                # ビルド成果物 (自動生成されるため省略)
└── tests/                 # 統合テスト
    └── api_test.rs        # APIのテストコード
```
