# Pasta Runtime Scripts

pasta.dll に同梱される標準ランタイムスクリプトです。

このフォルダーのファイルはビルド時に zip へ固めて pasta.dll に埋め込まれ、ゴーストの起動時に `profile/pasta/pasta_scripts/` へ自己展開されます。
ゴースト開発者はこのフォルダーのファイルを編集しないでください。
動作の変更が必要な場合は `scripts/` フォルダーに同名ファイルを配置することで上書きできます。
