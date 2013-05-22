namespace NeoDatis.Odb.Impl.Core.Oid
{
	[System.Serializable]
	public class DatabaseIdImpl : NeoDatis.Odb.DatabaseId
	{
		private long[] ids;

		public DatabaseIdImpl() : base()
		{
		}

		public DatabaseIdImpl(long[] ids) : base()
		{
			this.ids = ids;
		}

		public virtual long[] GetIds()
		{
			return ids;
		}

		public virtual void SetIds(long[] ids)
		{
			this.ids = ids;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < ids.Length; i++)
			{
				if (i != 0)
				{
					buffer.Append("-");
				}
				buffer.Append(ids[i].ToString());
			}
			return buffer.ToString();
		}

		public static NeoDatis.Odb.DatabaseId FromString(string sid)
		{
			string[] tokens = NeoDatis.Tool.Wrappers.OdbString.Split(sid, "-");
			long[] ids = new long[tokens.Length];
			for (int i = 0; i < ids.Length; i++)
			{
				ids[i] = long.Parse(tokens[i]);
			}
			return new NeoDatis.Odb.Impl.Core.Oid.DatabaseIdImpl(ids);
		}

		public override bool Equals(object @object)
		{
			if (@object == null || @object.GetType() != typeof(NeoDatis.Odb.Impl.Core.Oid.DatabaseIdImpl
				))
			{
				return false;
			}
			NeoDatis.Odb.Impl.Core.Oid.DatabaseIdImpl dbId = (NeoDatis.Odb.Impl.Core.Oid.DatabaseIdImpl
				)@object;
			for (int i = 0; i < ids.Length; i++)
			{
				if (ids[i] != dbId.ids[i])
				{
					return false;
				}
			}
			return true;
		}
	}
}
