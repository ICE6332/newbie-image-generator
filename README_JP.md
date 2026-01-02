<h1 align="center">Newbie Image Generator</h1>

<p align="center">
  <a href="README.md">中文</a> | <a href="README_EN.md">English</a> | <a href="README_JP.md">日本語</a>
</p>

<p align="center"><b>NewBie image Exp0.1</b> モデル専用のモダンな ComfyUI Web フロントエンド</p>

<p align="center">
  <img src="https://img.shields.io/badge/React-19-61DAFB?logo=react" alt="React">
  <img src="https://img.shields.io/badge/Rust-Axum-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Vite-7-646CFF?logo=vite" alt="Vite">
  <img src="https://img.shields.io/badge/Bun-Package%20Manager-000000?logo=bun" alt="Bun">
  <img src="https://img.shields.io/badge/shadcn/ui-000000?logo=shadcnui" alt="shadcn/ui">
</p>

<p align="center">
  <img src="assets/preview.png" alt="Preview" width="900">
</p>

## NewBie image Exp0.1 について

[NewBie image Exp0.1](https://huggingface.co/NewBie-AI/NewBie-image-Exp0.1) は Next-DiT アーキテクチャに基づく 3.5B パラメータのテキストから画像生成モデルで、高品質なアニメスタイルの画像生成に特化しています。

- **Text Encoder**: Gemma3-4B-it + Jina CLIP v2
- **VAE**: FLUX.1-dev 16channel VAE
- **特徴**: XML 構造化プロンプトをサポートし、複数キャラクターシーンの生成がより正確

## 機能

- **デュアルモード切替** - シンプルモードで直接プロンプト入力、構造化モードでキャラクター属性を視覚的に編集
- **XML 自動生成** - XML を手書きする必要なし、プロンプト内容に集中可能
- **XML インポート** - 既存の XML プロンプトをインポートし、自動解析して編集可能なフォームに変換
- **リアルタイムプレビュー** - 生成進捗をリアルタイム表示
- **高解像度修復** - 二次アップスケールで解像度と品質を向上
- **ワンクリックモデルダウンロード** - aria2c マルチスレッドダウンロード内蔵、レジューム対応
- **シンプルで美しい** - モダンな UI デザイン、直感的な操作
- **テーマ切替** - ダーク/ライトテーマ
- **レスポンシブレイアウト** - デスクトップとモバイル対応

### 従来の SDXL との違いと利点

- **プロンプト構造**: 従来の SDXL は自然言語/タグが主流。本プロジェクトは **XML 構造化プロンプト** を中心に、複数キャラクター、階層的な説明、シーン構成に最適化
- **モデル適応**: NewBie image Exp0.1 はアニメスタイルに最適化されており、対応するワークフローとパラメータのデフォルト値を内蔵
- **制御性**: 構造化タグでキャラクター属性/アクション/位置を明示的に分離し、キャラクター間の情報がより明確に
- **フロントエンド体験**: 視覚的なフォーム、XML インポート、リアルタイムプレビュー、接続検出を提供
- **多言語サポート**: Google Gemma3 モデルにより、中国語、英語、日本語でプロンプトを記述可能
- **適用シーン**: SDXL は汎用的なリアル/スタイライズ向け。NewBie + 本フロントエンドは **アニメ画像と複数キャラクター構成** に最適

## 前提条件

- **ComfyUI**: バージョン 0.7.0 以上が必要（NewBie ノードサポート内蔵）
- **NewBie image Exp0.1 モデル**: 完全なモデルファイルのダウンロードが必要（下記「モデルの準備」参照）

## クイックスタート

### 1. インストールパッケージのダウンロード

[Releases](../../releases) から最新版の圧縮ファイルをダウンロードし、任意のディレクトリに解凍。

### 2. モデルの準備

NewBie image Exp0.1 モデルをまだお持ちでない場合、以下の方法で取得できます：

**方法1: ダウンロードスクリプトを使用（推奨）**

インストールパッケージ内の `download_models.bat` を実行。マルチスレッドダウンロードとレジュームをサポート。

**方法2: 手動ダウンロード**

HuggingFace からモデルファイルをダウンロード：
| ファイル | ダウンロードリンク | 配置場所 |
|------|----------|----------|
| gemma3-4b-it.safetensors | [ダウンロード](https://huggingface.co/NewBie-AI/NewBie-image-Exp0.1/resolve/main/text_encoder/gemma3-4b-it.safetensors) | `ComfyUI/models/clip/` |
| jina-clip-v2.safetensors | [ダウンロード](https://huggingface.co/NewBie-AI/NewBie-image-Exp0.1/resolve/main/clip_model/jina-clip-v2.safetensors) | `ComfyUI/models/clip/` |
| VAE (newbie-image.safetensors) | [ダウンロード](https://huggingface.co/NewBie-AI/NewBie-image-Exp0.1/resolve/main/vae/diffusion_pytorch_model.safetensors) | `ComfyUI/models/vae/` |
| UNet (transformer) | [ダウンロード](https://huggingface.co/NewBie-AI/NewBie-image-Exp0.1/resolve/main/transformer/diffusion_pytorch_model.safetensors) | `ComfyUI/models/unet/` |

### 3. アプリケーションの起動

1. ComfyUI が実行中であることを確認（デフォルト `127.0.0.1:8188`）
2. `start.bat` をダブルクリック
3. ブラウザで http://localhost:3000 にアクセス

## 開発者ガイド

```bash
# リポジトリをクローン
git clone https://github.com/your-username/newbie-image-generator.git
cd newbie-image-generator

# バックエンド
cd backend
cp .env.example .env
cargo run

# フロントエンド (Vite)
cd ../frontend
bun install
bun run dev
```

**ローカル開発のヒント**
- バックエンドデフォルト: http://localhost:3000
- フロントエンドデフォルト: http://localhost:3001
- ComfyUI アドレスは UI で変更可能、フロントエンドとバックエンドは自動同期

## 設定

`backend/.env` を編集：

```env
HOST=127.0.0.1
PORT=3000
COMFYUI_HOST=127.0.0.1
COMFYUI_PORT=8188
ALLOW_REMOTE_COMFYUI=lan  # false|lan|true
```

## 技術スタック

| レイヤー | 技術 |
|------|------|
| Frontend | React 19, Vite 7, shadcn/ui, Bun |
| Backend | Rust, Axum, Tokio, WebSocket |

## 関連リンク

- [ComfyUI](https://github.com/comfyanonymous/ComfyUI)
- [NewBie image Exp0.1 モデル](https://huggingface.co/NewBie-AI/NewBie-image-Exp0.1)
- [ComfyUI-NewBie ノード](https://github.com/E-Anlia/ComfyUI-NewBie)
- [LoRA トレーナー](https://github.com/NewBieAI-Lab/NewbieLoraTrainer)
- [使用ガイド (中国語)](https://ai.feishu.cn/wiki/P3sgwUUjWih8ZWkpr0WcwXSMnTb)

## License

MIT
