using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Threading.Tasks.Dataflow;

namespace Pasta.API
{
    /// <summary>
    /// ログ収集モジュールのファクトリ。
    /// </summary>
    public interface IPastaGleanerFactory
    {
        /// <summary>収集モジュールの名前</summary>
        string GleanerName { get; }

        /// <summary>
        /// 与えられた設定パラメータにより、ログ収集モジュールを作成します。
        /// </summary>
        /// <param name="setting"></param>
        /// <returns></returns>
        IPastaGleaner CreateGleaner(string setting);
    }
}
