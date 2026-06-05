# 3言語のエンジニアに馴染みやすいように、シンプルなコマンド名にしています

.PHONY: up down restart status logs test-e2e fmt lint test check help

# 1. インフラの起動 (バックグラウンド)
up:
	docker compose up -d

# 2. コンテナの停止 ＋ ボリューム削除 ＋ システム全体のprune（ご要望の挙動）
down:
	docker compose down -v
	docker system prune -f --volumes

# 3. 完全初期化して再起動
restart: down up

# 4. コンテナの稼働ステータス確認
status:
	docker compose ps

# 5. リアルタイムログ監視
logs:
	docker compose logs -f

# 6. ローカルからの受入テスト（E2E）実行
test-e2e:
	DATABASE_URL=postgres://user:pass@localhost:5432/ev_db cargo test --test api_test

# コードの自動整形（rustfmt）
fmt:
	cargo fmt

# 静的解析（Clippy: CIと完全に同じ「警告をエラーとして扱う」厳格な設定）
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# テストの実行
test:
	cargo test

# コミット前の総チェック（フォーマット → リント → テスト を一気に実行）
check: fmt lint test

# ヘルプコマンド (make とだけ打った時に表示される)
help:
	@echo "利用可能なコマンド:"
	@echo "  make up       - インフラコンテナをバックグラウンドで起動"
	@echo "  make down     - コンテナを停止し、ボリューム削除＆pruneで完全クリーンアップ"
	@echo "  make restart  - 完全クリーンアップした後に再起動"
	@echo "  make status   - コンテナの稼働状況を確認"
	@echo "  make logs     - コンテナのログをストリーミング表示"
	@echo "  make test-e2e - ローカルからDockerのDBに向けてRustの受入テストを実行"
	@echo "  make fmt      - コードの自動整形（rustfmt）"
	@echo "  make lint     - 静的解析（Clippy）を�警告をエラーとして厳格に実行"
	@echo "  make test	 - Rustのユニットテストを実行"
	@echo "  make check	- フォーマット → リント → テスト を一気に実行"
	@echo "  make help     - このヘルプを表示"
