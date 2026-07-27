# 数理造形と表現演習

数理造形と表現演習の授業で作成したプログラムのリポジトリ。

## Web build

Cargo workspace 内の nannou アプリをまとめて Web 向けにビルドするには、リポジトリのルートで次を実行します。

```sh
./scripts/build-web-dist.sh
```

各アプリは独立して Trunk でビルドされ、成功したものだけが Cargo パッケージの配置に対応する `dist/` 以下のパスへ出力されます。たとえば `apps/attractor` は `dist/apps/attractor/index.html`、`apps/metaball/apps/exercise_a` は `dist/apps/metaball/apps/exercise_a/index.html` になります。

いずれかのアプリがコンパイルに失敗しても残りのビルドは継続され、失敗したアプリの中間生成物は `dist/` に含まれません。少なくとも1つ成功すればコマンドは成功として終了し、`dist/index.html` には成功したアプリだけへのリンクが生成されます。すべて失敗した場合は終了コード `1` になります。

事前に `cargo`、`trunk`、`python3` と `wasm32-unknown-unknown` ターゲットが必要です。

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
