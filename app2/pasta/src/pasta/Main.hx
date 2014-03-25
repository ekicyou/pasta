package pasta;

import js.Browser;
import js.html.CanvasElement;
import js.html.CanvasRenderingContext2D;

/**
 * ...
 * @author dot-station
 */

class Main 
{
	
	static function main() 
	{
		new Main();
	}
	
	public function new ()
	{
		// ページが読み込まれるのを待機
		Browser.window.onload = init;
	}
	private function init(e)
	{
		// canvasタグを生成
		var canvas:CanvasElement = cast(Browser.document.createElement("canvas"));
		// body タグ直下に生成
		Browser.document.body.appendChild(canvas);
		
		// canvasのコンテキストを取得
		var ctx:CanvasRenderingContext2D = canvas.getContext("2d");
		
		// 矩形を描く
		ctx.beginPath();
		ctx.fillRect(20, 20, 80, 40);
	}	
}