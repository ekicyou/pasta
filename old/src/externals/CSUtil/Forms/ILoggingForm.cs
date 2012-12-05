using System;
using System.ComponentModel;
using System.Drawing;
using System.Windows.Forms;

namespace CSUtil.Forms
{

    /// <summary>
    /// ロギングを行うFormで共通に利用するインターフェースを定義します。
    /// </summary>
    public interface ILoggingForm : IRmsLogging, IBaseLoggingForm
    {
    }

    /// <summary>
    /// RMSロギング処理を行っているモジュールのインターフェースです。
    /// </summary>
    public interface IRmsLogging
    {
        /// <summary>
        /// ログバッファを一旦吐き出し、ログファイルを閉じます。
        /// </summary>
        void OnWriteCloseLog();
    }

    /// <summary>
    /// ロギングを行うフォームのForm互換インターフェースを定義します。
    /// </summary>
    public interface IBaseLoggingForm
      : IContainerControl, IDropTarget, ISynchronizeInvoke, IWin32Window, IBindableComponent, IComponent, IDisposable
    {
        /// <summary>
        /// フォームのウィンドウ状態を取得または設定します。
        /// </summary>
        FormWindowState WindowState { get; set; }

        /// <summary>
        /// コントロールを z オーダーの最前面へ移動します。
        /// </summary>
        void BringToFront();

        /// <summary>
        /// フォームが初めて表示される直前に発生します。 
        /// </summary>
        event EventHandler Load;

        /// <summary>
        /// フォームのアイコンを取得または設定します。 
        /// </summary>
        Icon Icon { get; set; }

        /// <summary>
        /// コントロールのレイアウト ロジックを一時的に中断します。
        /// </summary>
        void SuspendLayout();

        /// <summary>
        /// 通常のレイアウト ロジックを再開します。 
        /// </summary>
        void ResumeLayout();

        /// <summary>
        /// 対象となるフォームの現在のマルチ ドキュメント インターフェイス (MDI) 親フォームを取得または設定します。 
        /// </summary>
        Form MdiParent { get; set; }

        /// <summary>
        /// ハンドルおよび子コントロールの作成を含めて、強制的にコントロールを作成します。
        /// </summary>
        void CreateControl();

        /// <summary>
        /// コントロールをユーザーに対して表示します。 
        /// </summary>
        void Show();

        /// <summary>
        /// フォームがコード内またはユーザーの操作によってアクティブになると発生します。
        /// </summary>
        event EventHandler Activated;

        /// <summary>
        /// フォームが閉じる前に発生します。
        /// </summary>
        event FormClosingEventHandler FormClosing;
    }

}