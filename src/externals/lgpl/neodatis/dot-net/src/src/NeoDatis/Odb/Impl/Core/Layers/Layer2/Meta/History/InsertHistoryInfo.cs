namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.History
{
	public class InsertHistoryInfo : NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.History.IHistoryInfo
	{
		private string type;

		private long position;

		private NeoDatis.Odb.OID oid;

		private NeoDatis.Odb.OID next;

		private NeoDatis.Odb.OID prev;

		public InsertHistoryInfo(string type, NeoDatis.Odb.OID oid, long position, NeoDatis.Odb.OID
			 prev, NeoDatis.Odb.OID next) : base()
		{
			this.type = type;
			this.position = position;
			this.oid = oid;
			this.next = next;
			this.prev = prev;
		}

		public virtual NeoDatis.Odb.OID GetNext()
		{
			return next;
		}

		public virtual long GetPosition()
		{
			return position;
		}

		public virtual NeoDatis.Odb.OID GetPrev()
		{
			return prev;
		}

		public virtual string GetType()
		{
			return type;
		}

		public virtual NeoDatis.Odb.OID GetOid()
		{
			return oid;
		}

		public override string ToString()
		{
			return type + " - oid=" + oid + " - pos=" + position + " - prev=" + prev + " - next="
				 + next;
		}
	}
}
