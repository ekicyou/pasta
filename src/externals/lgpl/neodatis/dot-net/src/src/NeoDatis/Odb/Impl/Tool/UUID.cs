namespace NeoDatis.Odb.Impl.Tool
{
	/// <summary>Unique ID generator</summary>
	/// <author>osmadja</author>
	public class UUID
	{
		public static long GetUniqueId(string simpleSeed)
		{
			lock (typeof(UUID))
			{
				long id = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs() - (long)(NeoDatis.Tool.Wrappers.OdbRandom
					.GetRandomDouble() * simpleSeed.GetHashCode());
				return id;
			}
		}

		public static long GetRandomLongId()
		{
			lock (typeof(UUID))
			{
				long id = (long)(NeoDatis.Tool.Wrappers.OdbRandom.GetRandomDouble() * long.MaxValue
					);
				return id;
			}
		}

		/// <summary>Returns a block marker , 5 longs</summary>
		/// <param name="position"></param>
		/// <returns>A 4 long array</returns>
		public static long[] GetBlockMarker(long position)
		{
			lock (typeof(UUID))
			{
				long l1 = unchecked((int)(0xFFEFCFBF));
				long[] id = new long[] { l1, l1, l1, position, l1 };
				return id;
			}
		}

		/// <summary>Returns a database id : 4 longs</summary>
		/// <param name="creationDate"></param>
		/// <returns>a 4 long array</returns>
		public static NeoDatis.Odb.DatabaseId GetDatabaseId(long creationDate)
		{
			lock (typeof(UUID))
			{
				long[] id = new long[] { creationDate, GetRandomLongId(), GetRandomLongId(), GetRandomLongId
					() };
				// FIXME do  not instanciate directly
				NeoDatis.Odb.DatabaseId databaseId = new NeoDatis.Odb.Impl.Core.Oid.DatabaseIdImpl
					(id);
				return databaseId;
			}
		}
	}
}
