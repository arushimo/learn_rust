# 3言語のエンジニアに馴染みやすいように、シンプルなコマンド名にしています

.PHONY: up down restart status logs test-e2e help

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

# ヘルプコマンド (make とだけ打った時に表示される)
help:
	@echo "利用可能なコマンド:"
	@echo "  make up       - インフラコンテナをバックグラウンドで起動"
	@echo "  make down     - コンテナを停止し、ボリューム削除＆pruneで完全クリーンアップ"
	@echo "  make restart  - 完全クリーンアップした後に再起動"
	@echo "  make status   - コンテナの稼働状況を確認"
	@echo "  make logs     - コンテナのログをストリーミング表示"
	@echo "  make test-e2e - ローカルからDockerのDBに向けてRustの受入テストを実行"