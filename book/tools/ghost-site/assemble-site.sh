#!/usr/bin/env bash
# hello-pasta の更新の置き場所を 1 回で組み立てる（hello-pasta-book-sync）。CI（manual.yml）と手元で同じものを使う。
#
# 使い方（リポジトリのルートで）:
#   bash book/tools/ghost-site/assemble-site.sh <commit> <tag> <release-nar> <out-dir>
#
# 前提: refs/tags/<tag> が手元に在る。pasta_check が PATH に在る（環境変数 PASTA_CHECK で上書きできる）。
#       <out-dir> は無いか空。
#
# 手順:
#   1. リポジトリの側: <commit> の hello-pasta の配下を core.autocrlf=true の git archive で取り出す（文字のファイルは CRLF）
#   2. nar の側: <release-nar> のうち <tag> のツリーに無いファイルを ghost-site.mjs nar-only で取り出す
#   3. pasta_check release --target <リポジトリの側> --copy <nar の側> で重ね・検査・updates.txt の作り直しを 1 回で行う
#   4. 重ねた nar を ghost-site.mjs build で照合し、入口のページを添えて <out-dir> に置く
#
# delete.txt は作らない（消した・改名したファイルは置かないだけにする）。
# updates.txt の作り手は pasta_check release だけなので、別の作り手との一致のテストは要らない。
# どの段の失敗も終了コード 1。失敗したときは <out-dir> を作りかけで残しうる（呼び手は公開しない）。
set -euo pipefail
trap 'exit 1' ERR

if [ "$#" -ne 4 ]; then
  echo "使い方: bash book/tools/ghost-site/assemble-site.sh <commit> <tag> <release-nar> <out-dir>" >&2
  exit 1
fi
commit=$1 tag=$2 nar=$3 out=$4
here=$(dirname "$0")
ghost=crates/pasta_sample_ghost/ghosts/hello-pasta
pasta_check=${PASTA_CHECK:-pasta_check}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/repo"

# 除外の 3 つの名前は update_files.rs の EXCLUDED_FILES と同じ（ghost-site.mjs の EXCLUDED_NAMES にも同じものが在る）。
# GNU tar の --exclude はどの階層でも最後の部品に当たる。
git -c core.autocrlf=true archive --format=tar "$commit" -- "$ghost/" |
  tar -x -C "$tmp/repo" --strip-components=4 \
    --exclude=updates.txt --exclude=updates2.dau --exclude=developer_options.txt
if [ -z "$(find "$tmp/repo" -type f -print -quit)" ]; then
  echo "NG $commit: $ghost の配下から 1 本も取り出せない" >&2
  exit 1
fi

node "$here/ghost-site.mjs" nar-only --nar "$nar" --tree "refs/tags/$tag:$ghost" --out "$tmp/nar"

"$pasta_check" release --target "$tmp/repo" --copy "$tmp/nar" --release "$tmp/merged" --nar "$tmp/merged.nar"

node "$here/ghost-site.mjs" build --nar "$tmp/merged.nar" --out "$out" --index "$here/index.html"
