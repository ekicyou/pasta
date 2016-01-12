using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace System.Xml.Linq
{
    public static class XmlExtensions
    {
        /// <summary>
        /// ドキュメントの内容をコピーします。
        /// 一旦テキストに変換し、読み込み直します。
        /// </summary>
        /// <param name="src"></param>
        /// <returns></returns>
        public static XDocument DeepCopy(this XDocument src)
        {
            byte[] buf;
            using (var st = new MemoryStream())
            using (var w = new StreamWriter(st)) {
                src.Save(w, SaveOptions.DisableFormatting);
                buf = st.ToArray();
            }
            using (var st = new MemoryStream(buf))
            using (var r = new StreamReader(st)) {
                return XDocument.Load(r, LoadOptions.PreserveWhitespace);
            }
        }

        #region [GetOrCreateElementImpl]
        /// <summary>
        /// 指定されたエレメントを返します。
        /// 存在しない場合空のエレメントを作成して返します。
        /// </summary>
        /// <param name="xcont"></param>
        /// <param name="qName"></param>
        /// <returns></returns>
        private static XElement GetOrCreateElementImpl(XContainer xcont, IEnumerable<XName> xnames)
        {
            XElement last = null;
            foreach (var n in xnames) {
                last = xcont.Element(n);
                if (last == null) {
                    last = new XElement(n);
                    xcont.Add(last);
                }
                xcont = last;
            }
            if (last == null) return xcont as XElement;
            return last;
        }


        #endregion
        #region [GetOrCreateElement]
        /// <summary>
        /// 指定されたエレメントを返します。
        /// 存在しない場合空のエレメントを作成して返します。
        /// </summary>
        /// <param name="xcont"></param>
        /// <param name="xnames"></param>
        /// <returns></returns>
        public static XElement GetOrCreateElement(this XContainer xcont, IEnumerable<XName> xnames)
        {
            return GetOrCreateElementImpl(xcont, xnames);
        }

        /// <summary>
        /// 指定されたエレメントを返します。
        /// 存在しない場合空のエレメントを作成して返します。
        /// </summary>
        /// <param name="xcont"></param>
        /// <param name="xnames"></param>
        /// <returns></returns>
        public static XElement GetOrCreateElement(this XContainer xcont, params XName[] xnames)
        {
            return GetOrCreateElementImpl(xcont, xnames);
        }


        #endregion
        #region [GetOrDefaultAttrFromXElement]
        private static XAttribute GetOrDefaultAttrFromXElement(XElement el, object defvalue, XName name)
        {
            var attr = el.Attribute(name);
            if (attr == null) {
                if (defvalue != null) attr = new XAttribute(name, defvalue);
                else attr = new XAttribute(name, string.Empty);
                el.Add(attr);
            }
            return attr;
        }


        #endregion
        #region [GetOrDefaultAttr]
        /// <summary>
        /// 指定された属性を返します。
        /// 存在しない場合指定された値で属性を作成して返します。
        /// </summary>
        /// <param name="el"></param>
        /// <param name="defvalue"></param>
        /// <param name="name"></param>
        /// <returns></returns>
        public static XAttribute GetOrDefaultAttr(this XElement el, object defvalue, XName name)
        {
            return GetOrDefaultAttrFromXElement(el, defvalue, name);
        }

        /// <summary>
        /// 指定された属性を返します。
        /// 存在しない場合指定された値で属性を作成して返します。
        /// </summary>
        /// <param name="el"></param>
        /// <param name="defvalue"></param>
        /// <param name="name"></param>
        /// <returns></returns>
        public static XAttribute GetOrDefaultAttr(this XContainer xcont, object defvalue, params XName[] name)
        {
            var attrName = name.Last();
            var el = GetOrCreateElementImpl(xcont, name.Take(name.Length - 1));
            return GetOrDefaultAttrFromXElement(el, defvalue, attrName);
        }

        /// <summary>
        /// 指定された属性を返します。
        /// 存在しない場合指定された値で属性を作成して返します。
        /// </summary>
        /// <param name="el"></param>
        /// <param name="defvalue"></param>
        /// <param name="name"></param>
        /// <returns></returns>
        public static XAttribute GetOrDefaultAttr(this XContainer el, object defvalue, IEnumerable<XName> name)
        {
            return el.GetOrDefaultAttr(defvalue, name.ToArray());
        }


        #endregion
        #region [GetOrCreateAttr]
        /// <summary>
        /// 指定された属性を返します。
        /// 存在しない場合空の属性を作成して返します。
        /// </summary>
        /// <param name="el"></param>
        /// <param name="name"></param>
        /// <returns></returns>
        public static XAttribute GetOrCreateAttr(this XElement el, XName name)
        {
            return GetOrDefaultAttrFromXElement(el, string.Empty, name);
        }

        /// <summary>
        /// 指定された属性を返します。
        /// 存在しない場合空の属性を作成して返します。
        /// </summary>
        /// <param name="el"></param>
        /// <param name="name"></param>
        /// <returns></returns>
        public static XAttribute GetOrCreateAttr(this XContainer el, params XName[] name)
        {
            return el.GetOrDefaultAttr("", name);
        }

        /// <summary>
        /// 指定された属性を返します。
        /// 存在しない場合空の属性を作成して返します。
        /// </summary>
        /// <param name="el"></param>
        /// <param name="name"></param>
        /// <returns></returns>
        public static XAttribute GetOrCreateAttr(this XContainer el, IEnumerable<XName> name)
        {
            return el.GetOrDefaultAttr("", name);
        }


        #endregion
        #region [GetAttr]
        /// <summary>
        /// 指定された属性の文字列を返します。
        /// </summary>
        /// <param name="el"></param>
        /// <param name="name"></param>
        /// <returns></returns>
        public static string GetAttr(this XContainer el, XName name)
        {
            var attr = el.GetOrCreateAttr(name);
            return attr.Value;
        }

        /// <summary>
        /// 指定された属性の文字列を返します。
        /// 属性が存在しない場合、デフォルト値を返します。
        /// </summary>
        /// <param name="el"></param>
        /// <param name="name"></param>
        /// <param name="defvalue"></param>
        /// <returns></returns>
        public static string GetAttr(this XContainer el, XName name, object defvalue)
        {
            var attr = el.GetOrDefaultAttr(defvalue, name);
            return attr.Value;
        }


        #endregion
        #region [SetAttr]
        /// <summary>
        /// 指定した属性に文字列を設定します。
        /// </summary>
        /// <param name="value"></param>
        /// <param name="el"></param>
        /// <param name="name"></param>
        public static void SetAttr(this string value, XContainer el, XName name)
        {
            var attr = el.GetOrCreateAttr(name);
            attr.Value = value;
        }


        #endregion
        #region [E]
        /// <summary>
        /// 指定した名前と内容を持つ System.Xml.Linq.XElement クラスの新しいインスタンスを初期化します。
        /// </summary>
        /// <param name="name"></param>
        /// <param name="content"></param>
        /// <returns></returns>
        public static XElement E(this XName name, object content)
        {
            return new XElement(name, content);
        }

        /// <summary>
        /// 指定した名前と内容を持つ System.Xml.Linq.XElement クラスの新しいインスタンスを初期化します。
        /// </summary>
        /// <param name="name"></param>
        /// <param name="content"></param>
        /// <returns></returns>
        public static XElement E(this XName name, params object[] content)
        {
            return new XElement(name, content);
        }


        #endregion
        #region [A]
        /// <summary>
        /// 指定した名前と内容を持つ System.Xml.Linq.XAttribute クラスの新しいインスタンスを初期化します。
        /// </summary>
        /// <param name="name"></param>
        /// <param name="content"></param>
        /// <returns></returns>
        public static XAttribute A(this XName name, object content)
        {
            return new XAttribute(name, content);
        }

        /// <summary>
        /// 指定した名前と内容を持つ System.Xml.Linq.XAttribute クラスの新しいインスタンスを初期化します。
        /// </summary>
        /// <param name="name"></param>
        /// <param name="content"></param>
        /// <returns></returns>
        public static XAttribute A(this XName name, params object[] content)
        {
            return new XAttribute(name, content);
        }


        #endregion
    }
}