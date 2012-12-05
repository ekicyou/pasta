using System;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Text.RegularExpressions;

namespace CSUtil.Reflection
{
    /// <summary>
    /// アセンブリ操作ユーティリティ。
    /// </summary>
    public static class AssemblyUtil40
    {
        /// <summary>
        /// アセンブリパスの解決に、64bit環境かどうかに応じてこのDLLのパス+ x86 or x64 を追加します。
        /// dllのみ検索対象とします。
        /// </summary>
        public static void InitX86X64AssemblyResolver()
        {
            X86X64AssemblyResolver.Check();
        }

        private static class X86X64AssemblyResolver
        {
            public static void Check() { }


            static X86X64AssemblyResolver()
            {
                var baseDir = AssemblyUtil.GetCallingAssemblyDirctory();
                var cpu = Environment.Is64BitProcess ? "x64" : "x86";
                var dir = Path.Combine(baseDir, cpu);
                AppDomain.CurrentDomain.AssemblyResolve += (sender, ev) =>
                {
                    var name = ev.Name.Split(',')[0] + ".dll";
                    var path = Path.Combine(dir, name);
                    if(File.Exists(path)) {
                        return Assembly.LoadFrom(path);
                    }
                    return null;
                };


            }

        }

    }
}