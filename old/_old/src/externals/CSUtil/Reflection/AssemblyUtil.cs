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
    public static class AssemblyUtil
    {
        /// <summary>
        /// 指定されたアセンブリのPathを返します。
        /// 返すPathはシャドーコピーではなく、本体のPathです。
        /// </summary>
        /// <param name="assembly"></param>
        /// <returns></returns>
        public static string GetAssemblyPath(Assembly assembly)
        {
            Uri binURI = new Uri(assembly.CodeBase);
            string encPath = binURI.AbsolutePath;
            string path = Uri.UnescapeDataString(encPath);
            string fullPath = Path.GetFullPath(path);
            return fullPath;
        }

        /// <summary>
        /// 呼び出し元のアセンブリのPathを返します。
        /// 返すPathはシャドーコピーではなく、本体のPathです。
        /// </summary>
        /// <returns></returns>
        public static string GetCallingAssemblyPath()
        {
            return GetAssemblyPath(Assembly.GetCallingAssembly());
        }



        /// <summary>
        /// 指定されたアセンブリのDirecryPathを返します。
        /// 返すPathはシャドーコピーではなく、本体のPathです。
        /// </summary>
        /// <param name="assembly"></param>
        /// <returns></returns>
        public static string GetAssemblyDirctory(Assembly assembly)
        {
            return Path.GetDirectoryName(GetAssemblyPath(assembly));
        }

        /// <summary>
        /// 呼び出し元のアセンブリの DirecryPathを返します。
        /// 返すPathはシャドーコピーではなく、本体のPathです。
        /// </summary>
        /// <returns></returns>
        public static string GetCallingAssemblyDirctory()
        {
            return GetAssemblyDirctory(Assembly.GetCallingAssembly());
        }

        /// <summary>
        /// パスをアセンブリ参照対象に追加します。
        /// </summary>
        /// <param name="path"></param>
        public static void AddCurrentAssemblyResolvePath(string path)
        {
            AddCurrentAssemblyResolvePath_Deligate cb =
                new AddCurrentAssemblyResolvePath_Deligate(path);
            AppDomain.CurrentDomain.AssemblyResolve += cb.CallBack;
        }

        private class AddCurrentAssemblyResolvePath_Deligate
        {
            private readonly string dir;
            private readonly Regex re;

            private Assembly Find(string path, string name)
            {
                try {
                    Assembly assembly = Assembly.LoadFile(path);
                    //      if(assembly->FullName != name) return nullptr;
                    return assembly;
                }
                catch(Exception) {
                    return null;
                }
            }

            internal AddCurrentAssemblyResolvePath_Deligate(string path)
            {
                dir = Path.GetFullPath(path);
                re = new Regex("^([^,]*), Version=([^,]*), Culture=([^,]*), PublicKeyToken=(.*)$",
                    RegexOptions.Compiled);
            }

            internal Assembly CallBack(object sender, ResolveEventArgs args)
            {
                Debug.WriteLine(string.Format("AssemblyResolve: [{0}]", args.Name));

                Match m = re.Match(args.Name);
                if(!m.Success) return null;

                string assemblyName = m.Groups[1].Value;
                string version = m.Groups[2].Value;
                string culture = m.Groups[3].Value;
                string publicKeyToken = m.Groups[4].Value;

                Assembly rc;
                String path;

                path = string.Format("{0}\\{1}.dll", dir, assemblyName);
                rc = Find(path, args.Name);
                return rc;
            }
        }

        /// <summary>
        /// スタックを指定した階層を遡ってメソッド名を取得します。
        /// </summary>
        /// <param name="skipFrames">遡るカウント。０のとき呼び出したメソッド。</param>
        /// <returns></returns>
        public static string GetMethodName(int skipFrames)
        {
            StackFrame sf = new StackFrame(skipFrames + 1);
            MethodBase m = sf.GetMethod();
            return m.ReflectedType.Name + "::" + m.Name;
        }

        /// <summary>
        /// 呼び出し元メソッドのメソッド名を取得します。
        /// </summary>
        /// <returns></returns>
        public static string GetMethodName()
        {
            return GetMethodName(1);
        }

    }
}