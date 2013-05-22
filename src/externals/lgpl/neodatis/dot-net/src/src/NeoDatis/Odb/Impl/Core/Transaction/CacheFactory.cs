namespace NeoDatis.Odb.Impl.Core.Transaction
{
	public class CacheFactory
	{
		public static NeoDatis.Odb.Core.Transaction.ICache GetLocalCache(NeoDatis.Odb.Core.Transaction.ISession
			 session, string name)
		{
			if (NeoDatis.Odb.OdbConfiguration.UseLazyCache())
			{
				return null;
			}
			//new LazyCache(session);
			return new NeoDatis.Odb.Impl.Core.Transaction.Cache(session, name);
		}

		public static NeoDatis.Odb.Core.Transaction.ITmpCache GetLocalTmpCache(NeoDatis.Odb.Core.Transaction.ISession
			 session, string name)
		{
			if (NeoDatis.Odb.OdbConfiguration.UseLazyCache())
			{
				return new NeoDatis.Odb.Impl.Core.Transaction.TmpCache(session, name);
			}
			return new NeoDatis.Odb.Impl.Core.Transaction.TmpCache(session, name);
		}

		public static NeoDatis.Odb.Core.Transaction.ICache GetServerCache(NeoDatis.Odb.Core.Transaction.ISession
			 session)
		{
			if (NeoDatis.Odb.OdbConfiguration.UseLazyCache())
			{
				return null;
			}
			//new LazyServerCache(session);
			return new NeoDatis.Odb.Impl.Core.Transaction.ServerCache(session);
		}

		/// <summary>
		/// This factory method returns an implementation of
		/// <see cref="NeoDatis.Odb.Core.Transaction.ICrossSessionCache">NeoDatis.Odb.Core.Transaction.ICrossSessionCache
		/// 	</see>
		/// to take over the objects across the sessions.
		/// </summary>
		/// <param name="identification">TODO</param>
		/// <returns>
		/// 
		/// <see cref="NeoDatis.Odb.Core.Transaction.ICrossSessionCache">NeoDatis.Odb.Core.Transaction.ICrossSessionCache
		/// 	</see>
		/// </returns>
		public static NeoDatis.Odb.Core.Transaction.ICrossSessionCache GetCrossSessionCache
			(string identification)
		{
			return NeoDatis.Odb.Impl.Core.Transaction.CrossSessionCache.GetInstance(identification
				);
		}
	}
}
