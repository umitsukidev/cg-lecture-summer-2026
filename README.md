# 数理造形と表現演習

数理造形と表現演習の授業で作成したプログラムのリポジトリ。

## Web build

Cargo workspace 内の nannou アプリをまとめて Web 向けにビルドするには、リポジトリのルートで次を実行します。

```sh
./scripts/build-web-dist.sh
```

各アプリは独立して Trunk でビルドされ、成功したものだけが Cargo パッケージの配置に対応する `dist/` 以下のパスへ出力されます。たとえば `apps/attractor` は `dist/apps/attractor/index.html`、`apps/metaball/apps/exercise_a` は `dist/apps/metaball/apps/exercise_a/index.html` になります。

いずれかのアプリがコンパイルに失敗しても残りのビルドは継続され、失敗したアプリの中間生成物は `dist/` に含まれません。少なくとも1つ成功すればコマンドは成功として終了し、`dist/index.html` には成功したアプリだけへのリンクが生成されます。すべて失敗した場合は終了コード `1` になります。

成功した各アプリのすべての Wasm は、Cloudflare Workers Static Assets の1ファイル25 MiB制限に収まるよう、Brotli圧縮した内容で同じ `.wasm` ファイルへ置き換えられます。Wasmへのリクエストだけは `worker/index.js` が先に処理し、二重圧縮を避けながら `Content-Encoding: br` と `Content-Type: application/wasm` を設定します。圧縮後も25 MiBを超えるアプリや圧縮に失敗したアプリは `dist/` に含まれません。

ハッシュ付きの Wasm レスポンスはブラウザとCloudflare CDNで1年間キャッシュされます。Workers Cachingはデプロイバージョン間でも共有されるため、内容ハッシュが変わらないWasmは次回のデプロイ後もキャッシュを再利用します。

一括Webビルドが使用する `wasm-release` では、依存クレートを含む `log` と `tracing` の出力をコンパイル時にすべて除外します。通常のnative releaseビルドやdebugビルドのログレベルは変わりません。panicはログ機構とは別にブラウザコンソールへ出力されますが、Wasm内のローカルなホーム絶対パスは `source` に置換されます。置換前の絶対パスが残ったWasmは `dist/` に含まれません。

このため `dist/` を通常の静的ファイルサーバーで直接配信することはできません。Cloudflareへのデプロイ前にローカル確認する場合は `pnpm exec wrangler dev` を使用してください。

事前に `brotli`、`cargo`、`trunk`、`python3` と `wasm32-unknown-unknown` ターゲットが必要です。

## License

The source code in this repository is licensed under the [MIT License](LICENSE).

> [!WARNING]
> **Important Notice Regarding Images / 画像に関する重要なお知らせ**
>
> - **English:** Please note that all image files located in the `/apps/metaball/apps/exercise_c/assets/images/` directory are **NOT** covered by the MIT license. All rights are reserved by the author. Copying, redistributing, modifying, or using these images for any purpose without explicit permission is strictly prohibited.
> - **日本語:** `/apps/metaball/apps/exercise_c/assets/images/` ディレクトリ内のすべての画像ファイルは、MITライセンスの**適用外**となります。著作者がすべての権利を保有しており、事前の許可なくこれらを複製、再配布、改変、または目的を問わず使用することは固く禁止します。

## Fonts / フォント

- **Noto Sans JP**: `apps/stable_fluids/assets/` に含まれるフォントファイルは、[SIL Open Font License 1.1](apps/stable_fluids/assets/OFL.txt) に基づいてライセンスされています。
  - Copyright 2020-2022 The Noto Project Authors (https://github.com/notofonts/noto-cjk)
  - Copyright 2014-2021 Adobe (http://www.adobe.com/), with Reserved Font Name 'Source'.
