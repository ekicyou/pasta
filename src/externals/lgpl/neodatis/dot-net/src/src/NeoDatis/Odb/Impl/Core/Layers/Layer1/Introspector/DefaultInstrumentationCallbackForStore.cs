namespace NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector
{
	/// <author>olivier</author>
	public class DefaultInstrumentationCallbackForStore : NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
	{
		protected bool isUpdate;

		protected NeoDatis.Odb.Core.Trigger.ITriggerManager triggerManager;

		protected NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache;

		protected NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine;

		public DefaultInstrumentationCallbackForStore(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine, NeoDatis.Odb.Core.Trigger.ITriggerManager triggerManager, bool isUpdate
			) : base()
		{
			this.engine = engine;
			this.triggerManager = triggerManager;
			this.isUpdate = isUpdate;
			// Just for junits
			if (engine != null)
			{
				this.crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetCrossSessionCache
					(engine.GetBaseIdentification().GetIdentification());
			}
		}

		public virtual bool ObjectFound(object @object)
		{
			if (!isUpdate)
			{
				if (triggerManager != null)
				{
					triggerManager.ManageInsertTriggerBefore(@object.GetType().FullName, @object);
				}
			}
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				CheckIfObjectMustBeReconnected(@object);
			}
			return true;
		}

		/// <summary>
		/// Used to check if object must be reconnected to current session
		/// <pre>
		/// An object must be reconnected to session if OdbConfiguration.reconnectObjectsToSession() is true
		/// and object is not in local cache and is in cross session cache.
		/// </summary>
		/// <remarks>
		/// Used to check if object must be reconnected to current session
		/// <pre>
		/// An object must be reconnected to session if OdbConfiguration.reconnectObjectsToSession() is true
		/// and object is not in local cache and is in cross session cache. In this case
		/// we had it to local cache
		/// </pre>
		/// </remarks>
		/// <param name="@object"></param>
		private void CheckIfObjectMustBeReconnected(object @object)
		{
			if (engine == null)
			{
				// This protection is for JUnit
				return;
			}
			NeoDatis.Odb.Core.Transaction.ISession session = engine.GetSession(true);
			// If object is in local cache, no need to reconnect it
			if (session.GetCache().ExistObject(@object))
			{
				return;
			}
			NeoDatis.Odb.OID oidCrossSession = crossSessionCache.GetOid(@object);
			if (oidCrossSession != null)
			{
				// reconnect object
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = engine.GetObjectInfoHeaderFromOid
					(oidCrossSession);
				session.AddObjectToCache(oidCrossSession, @object, oih);
			}
		}
	}
}
