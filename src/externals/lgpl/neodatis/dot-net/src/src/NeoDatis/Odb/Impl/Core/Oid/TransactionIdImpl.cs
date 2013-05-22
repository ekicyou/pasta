namespace NeoDatis.Odb.Impl.Core.Oid
{
	[System.Serializable]
	public class TransactionIdImpl : NeoDatis.Odb.TransactionId
	{
		private long id1;

		private long id2;

		private NeoDatis.Odb.DatabaseId databaseId;

		public TransactionIdImpl(NeoDatis.Odb.DatabaseId databaseID, long id1, long id2) : 
			base()
		{
			this.databaseId = databaseID;
			this.id1 = id1;
			this.id2 = id2;
		}

		public virtual long GetId1()
		{
			return id1;
		}

		public virtual NeoDatis.Odb.DatabaseId GetDatabaseId()
		{
			return databaseId;
		}

		public virtual long GetId2()
		{
			return id2;
		}

		public virtual NeoDatis.Odb.TransactionId Next()
		{
			return new NeoDatis.Odb.Impl.Core.Oid.TransactionIdImpl(databaseId, id1, id2 + 1);
		}

		public virtual NeoDatis.Odb.TransactionId Prev()
		{
			return new NeoDatis.Odb.Impl.Core.Oid.TransactionIdImpl(databaseId, id1, id2 - 1);
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder("tid=").Append(id1
				.ToString()).Append(id2.ToString());
			buffer.Append(" - dbid=").Append(databaseId);
			return buffer.ToString();
		}

		public override bool Equals(object @object)
		{
			if (@object == null || @object.GetType() != typeof(NeoDatis.Odb.Impl.Core.Oid.TransactionIdImpl
				))
			{
				return false;
			}
			NeoDatis.Odb.Impl.Core.Oid.TransactionIdImpl tid = (NeoDatis.Odb.Impl.Core.Oid.TransactionIdImpl
				)@object;
			return id1 == tid.id1 && id2 == tid.id2 && databaseId.Equals(tid.databaseId);
		}
	}
}
