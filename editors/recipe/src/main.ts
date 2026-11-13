import "./style.css";
import { createEditor, monaco } from "./editor.ts";

// shortcut: 関門（2.3）で色の重なりを見るための仮の文書。4.2 の仮の配線（起動の引数のゴーストを開く）で置き換える
const SAMPLE = [
  "＃ 色の重なりの確かめ",
  "＠単語：りんご、みかん",
  "",
  "＊挨拶",
  "```lua",
  "function SCENE.次の話(act)",
  '    local n = 42 -- コメント',
  '    return "とっておき"',
  "end",
  "```",
  "　女の子：こんにちは、＠単語　をどうぞ。",
  "　＞＠次の話（）",
  "",
  "　・朝",
  "　　男の子：おはよう！",
  "",
].join("\n");

const editor = await createEditor(document.getElementById("editor")!);
editor.setModel(monaco.editor.createModel(SAMPLE, "pasta", monaco.Uri.file("sample.pasta")));
