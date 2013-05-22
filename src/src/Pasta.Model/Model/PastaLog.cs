using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using ProtoBuf;

namespace Pasta.Model
{
    /// <summary>
    /// Pasta ログ情報
    /// </summary>
    [ProtoContract]
    public sealed class PastaLog : TimeIdObject
    {
        #region ＤＢプロパティ：
        /// <summary>ログ生成アプリ</summary>
        [ProtoMember(1)]
        public string Application { get { return _Application; } set { _Application.Set(value, this); } }
        private NotificationStore<string> _Application;


        /// <summary>未解析データ</summary>
        [ProtoMember(2)]
        public byte[] Raw { get { return _Raw; } set { _Raw.Set(value, this); } }
        private NotificationStore<byte[]> _Raw;



        /// <summary>行為の実行者</summary>
        [ProtoMember(3)]
        public string Sender { get { return _Sender; } set { _Sender.Set(value, this); } }
        private NotificationStore<string> _Sender;

        /// <summary>実行者の所属</summary>
        [ProtoMember(4)]
        public string SenderGroup { get { return _SenderGroup; } set { _SenderGroup.Set(value, this); } }
        private NotificationStore<string> _SenderGroup;



        /// <summary>行為</summary>
        [ProtoMember(5)]
        public string Action { get { return _Action; } set { _Action.Set(value, this); } }
        private NotificationStore<string> _Action;

        /// <summary>行為の所属</summary>
        [ProtoMember(6)]
        public string ActionGroup { get { return _ActionGroup; } set { _ActionGroup.Set(value, this); } }
        private NotificationStore<string> _ActionGroup;

        /// <summary>行為の詳細</summary>
        [ProtoMember(7)]
        public string ActionDescription { get { return _ActionDescription; } set { _ActionDescription.Set(value, this); } }
        private NotificationStore<string> _ActionDescription;



        /// <summary>対象</summary>
        [ProtoMember(8)]
        public string Target { get { return _Target; } set { _Target.Set(value, this); } }
        private NotificationStore<string> _Target;

        /// <summary>対象の所属</summary>
        [ProtoMember(9)]
        public string TargetGroup { get { return _TargetGroup; } set { _TargetGroup.Set(value, this); } }
        private NotificationStore<string> _TargetGroup;



        /// <summary>行為に使ったアイテム</summary>
        [ProtoMember(10)]
        public string Item { get { return _Item; } set { _Item.Set(value, this); } }
        private NotificationStore<string> _Item;



        /// <summary>影響を受ける属性</summary>
        [ProtoMember(11)]
        public string Property { get { return _Property; } set { _Property.Set(value, this); } }
        private NotificationStore<string> _Property;


        /// <summary>結果（文字表現）</summary>
        [ProtoMember(12)]
        public string ResultText { get { return _ResultText; } set { _ResultText.Set(value, this); } }
        private NotificationStore<string> _ResultText;

        /// <summary>結果（数値表現）</summary>
        [ProtoMember(13)]
        public double ResultValue { get { return _ResultValue; } set { _ResultValue.Set(value, this); } }
        private NotificationStore<double> _ResultValue;


        /// <summary>ある行動に対する副次効果の場合、発生元</summary>
        [ProtoMember(14)]
        public long SourceID { get { return _SourceID; } set { _SourceID.Set(value, this); } }
        private NotificationStore<long> _SourceID;


        #endregion
        #region 参照プロパティ：

        /// <summary>副次効果の場合、発生元</summary>
        public PastaLog Source { get { return _Source; } internal set { _Source.Set(value, this); } }
        private NotificationStore<PastaLog> _Source;

        #endregion

    }
}