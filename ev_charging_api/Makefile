# 3言語のエンジニアに馴染みやすいように、シンプルなコマンド名にしています

.PHONY: up down restart status logs test-e2e fmt lint test check help k8s-start k8s-build k8s-deploy k8s-up k8s-down k8s-logs k8s-pf-api k8s-pf-jaeger

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

# 1. Minikubeクラスターの起動
k8s-start:
	minikube start

# 2. Minikube内部のDockerに向かってイメージをビルド
# 💡 Makeでは1行ごとに別々のシェルが立ち上がるため、同じ行で eval と docker build を繋ぎます（$ は $$ にエスケープ）
k8s-build:
	@echo "Minikube内にイメージをビルドしています..."
	eval $$(minikube docker-env) && docker build -t ev_charging_api:dev .

# 3. Kustomizeによるマニフェスト適用
k8s-deploy:
	kubectl apply -k infra/k8s/overlays/dev/

# 🚀 上記1〜3を全自動で一気に実行する「魔法のコマンド」
k8s-up: k8s-start k8s-build k8s-deploy
	@echo "デプロイ完了！PodがRunningになるまで 'kubectl get pods -w' で確認してください。"

# 環境のお掃除（リソースの削除）
k8s-down:
	kubectl delete -k infra/k8s/overlays/dev/

# APIコンテナのログをリアルタイム監視（-f: tail 相当）
k8s-logs:
	kubectl logs -l app=ev-charging-api -f

# 🔗 ポートフォワード（別々のターミナルで実行する用）
k8s-pf-api:
	kubectl port-forward svc/ev-charging-api-svc 50051:50051

k8s-pf-jaeger:
	kubectl port-forward svc/jaeger-svc 16686:16686

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
	@echo "  make k8s-start   - Minikubeクラスターを起動"
	@echo "  make k8s-build   - Minikube内部のDockerに向かってイメージをビルド"
	@echo "  make k8s-deploy  - Kustomizeによるマニフェスト適用"
	@echo "  make k8s-up      - 上記3ステップを全自動で一気に実行"
	@echo "  make k8s-down    - Kustomizeで作成したリソースを削除"
	@echo "  make k8s-logs    - APIコンテナのログをリアルタイム監視"
	@echo "  make k8s-pf-api   - APIコンテナのポートフォワード（ローカル50051 → クラスター内50051）"
	@echo "  make k8s-pf-jaeger - Jaegerのポートフォワード（ローカル16686 → クラスター内16686）"
