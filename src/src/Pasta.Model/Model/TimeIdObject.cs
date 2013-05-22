using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using ProtoBuf;

namespace Pasta.Model
{
    [ProtoContract]
    [ProtoInclude(2, typeof(PastaLog))]
    public class TimeIdObject : NotificationObject
    {
        /// <summary>発生時刻(UTC)。</summary>
        public DateTime UTC { get { return _UTC; } private set { _UTC.Set(value, this); } }
        private NotificationStore<DateTime> _UTC;


        /// <summary>発生時刻ID(UTCに変換される値)。発生時刻はユニークKeyになるように調整する事</summary>
        [ProtoMember(1)]
        public long TimeID
        {
            get { return _TimeID; }
            set
            {
                if (!_TimeID.Set(value, this)) return;
                UTC = DateTime.FromBinary(TimeID);
            }
        }
        private NotificationStore<long> _TimeID;

        /// <summary>
        /// ユニークになるように時刻IDを生成します。
        /// </summary>
        public void SetUniqueTime() { TimeID =TimeIdUtil.GetUniqueTimeID(); }




    }
}