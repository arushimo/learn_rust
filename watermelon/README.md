# Watermelon

Rust/Wasm + Canvas API で動くスイカゲーム風プロトタイプです。

ゲームロジックと物理演算は Rust/Wasm 側で処理し、JavaScript は入力と描画だけを担当します。Tauri 2 の iOS/mobile 化も、物理演算を Tauri Rust backend に移さず **Wasm のまま**進める構成です。

## 構成

```text
.
├── Cargo.toml          # Wasm game logic crate
├── src/lib.rs          # GameState / Rapier2d / merge logic
├── index.html          # Canvas UI
├── main.js             # Wasm呼び出し + Canvas描画
├── vite.config.mjs     # Tauri mobile dev対応のVite設定
├── package.json        # npm scripts
└── src-tauri/          # Tauri 2 app wrapper
```

## 前提ツール

- Rust
- Node.js / npm
- wasm-pack
- Xcode
- Tauri 2 CLI は `package.json` の devDependency 経由で使用

`wasm-pack` がない場合:

```bash
cargo install wasm-pack
```

## Web版として起動

```bash
cd /Users/kawamuraakito/code/learn_rust/watermelon
npm install
npm run dev
```

`npm run dev` は先に Wasm をビルドしてから Vite を起動します。

```bash
npm run build:wasm
vite
```

## Production build

```bash
npm run build
```

このコマンドで以下を行います。

1. `wasm-pack build --target web --out-dir pkg`
2. `vite build`
3. `dist/` に Tauri が読み込む静的アセットを生成

Wasm は Vite によって `dist/assets/*.wasm` として同梱されます。

## Tauri Desktop build確認

```bash
npx tauri build --debug --bundles app
```

成功すると macOS app bundle が生成されます。

```text
src-tauri/target/debug/bundle/macos/Watermelon.app
```

通常の `npx tauri build --debug` は DMG も作ろうとします。DMG 作成で失敗する環境があるため、まずは `--bundles app` で Tauri wrapper と Wasm 同梱の確認をするのがおすすめです。

## Tauri iOS初期化

初回のみ実行します。

```bash
npm run ios:init
```

生成される Xcode project:

```text
src-tauri/gen/apple/app.xcodeproj
```

すでに生成済みの場合、通常は再実行不要です。

## iPhone実機デプロイ

Apple Developer Team ID を環境変数に設定します。

```bash
export APPLE_DEVELOPMENT_TEAM=XXXXXXXXXX
```

Xcode を開いて実機を選択する場合:

```bash
npm run ios:open
```

CLI から実機 dev 起動する場合:

```bash
npm run ios:dev
```

IPA build:

```bash
npm run ios:build
```

## iOS実機開発時の注意

`vite.config.mjs` は Tauri mobile dev 用に `TAURI_DEV_HOST` を見ています。

```js
const host = process.env.TAURI_DEV_HOST;
```

`tauri ios dev --host` 実行時、iPhone 実機の WebView は Mac 上の Vite dev server へ接続します。そのため、Mac と iPhone が同じネットワークにいる必要があります。

## Wasm読み込み設定

Wasm は `main.js` から以下のように読み込みます。

```js
import init, { GameState } from "./pkg/watermelon.js";

await init();
const game = new GameState();
```

`wasm-pack --target web` の出力は `new URL("watermelon_bg.wasm", import.meta.url)` を内部で使うため、Vite build 後も相対パスで解決されます。

Tauri の CSP では WebAssembly 実行のために `wasm-unsafe-eval` を許可しています。

```json
"security": {
  "csp": "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self' ipc: http://ipc.localhost; img-src 'self' data:; style-src 'self' 'unsafe-inline'",
  "devCsp": "default-src 'self' http: ws: ipc: http://ipc.localhost; script-src 'self' 'wasm-unsafe-eval' 'unsafe-eval'; connect-src 'self' http: ws: ipc: http://ipc.localhost; img-src 'self' data:; style-src 'self' 'unsafe-inline'"
}
```

## 既知の環境依存エラー

### Code signing warning

```text
No code signing certificates found.
```

実機デプロイには Apple Developer Team ID と signing 設定が必要です。

```bash
export APPLE_DEVELOPMENT_TEAM=XXXXXXXXXX
```

または `src-tauri/tauri.conf.json` の `bundle.iOS.developmentTeam` に設定します。

### Simulator SDK error

```text
Xcode Simulator SDK ... is not installed, please open Xcode
```

Xcode を開いて追加コンポーネントや Simulator SDK をインストールしてください。

## 現在確認済み

```bash
npm run build
npx tauri build --debug --bundles app
```

上記は成功確認済みです。
