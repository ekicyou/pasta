namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine
{
	public class ServerStorageEngine : NeoDatis.Odb.Core.Layers.Layer3.Engine.AbstractStorageEngine
		, NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine
	{
		private NeoDatis.Odb.Core.Server.Transaction.ISessionManager sessionManager;

		public ServerStorageEngine(NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification parameters
			) : base(parameters)
		{
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter BuildObjectWriter()
		{
			return provider.GetServerObjectWriter(this);
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IObjectReader BuildObjectReader()
		{
			return provider.GetServerObjectReader(this);
		}

		public override NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			BuildObjectIntrospector()
		{
			return provider.GetServerObjectIntrospector(this);
		}

		public override NeoDatis.Odb.Core.Trigger.ITriggerManager BuildTriggerManager()
		{
			return provider.GetServerTriggerManager(this);
		}

		/// <exception cref="System.IO.IOException"></exception>
		protected virtual NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface BuildFSI
			()
		{
			return new NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.ServerFileSystemInterface
				("data", baseIdentification, true, NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData
				());
		}

		public override NeoDatis.Odb.Core.Transaction.ISession GetSession(bool throwExceptionIfDoesNotExist
			)
		{
			return sessionManager.GetSession(baseIdentification.GetIdentification(), throwExceptionIfDoesNotExist
				);
		}

		public override NeoDatis.Odb.Core.Transaction.ISession BuildDefaultSession()
		{
			NeoDatis.Odb.Core.ICoreProvider provider = NeoDatis.Odb.OdbConfiguration.GetCoreProvider
				();
			if (sessionManager == null)
			{
				sessionManager = provider.GetClientServerSessionManager();
			}
			NeoDatis.Odb.Core.Transaction.ISession session = provider.GetServerSession(this, 
				"default");
			//FIXME Remove commented line
			//session.setBaseIdentification(((ServerFileParameter) this.getBaseIdentification()).getBaseName());
			return session;
		}

		public override void AddSession(NeoDatis.Odb.Core.Transaction.ISession session, bool
			 readMetamodel)
		{
			sessionManager.AddSession(session);
			base.AddSession(session, readMetamodel);
		}

		protected override NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel GetMetaModel()
		{
			return GetSession(true).GetMetaModel();
		}

		public override void Commit()
		{
			base.Commit();
		}

		/// <summary>Write an object meta-representation.</summary>
		/// <remarks>
		/// Write an object meta-representation. TODO Use a mutex to guarantee unique
		/// access to the file at this moment. This should be change
		/// </remarks>
		/// <param name="oid"></param>
		/// <param name="aoi"></param>
		/// <param name="position"></param>
		/// <param name="updatePointers"></param>
		/// <returns>The object position or id if negative</returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		public override NeoDatis.Odb.OID WriteObjectInfo(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, long position, bool updatePointers)
		{
			oid = base.WriteObjectInfo(oid, nnoi, position, updatePointers);
			return oid;
		}

		/// <summary>It is overiden to manage triggers</summary>
		public override void DeleteObjectWithOid(NeoDatis.Odb.OID oid)
		{
			// Check if oih is in the cache
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = GetSession(true).GetCache
				().GetObjectInfoHeaderFromOid(oid, false);
			if (oih == null)
			{
				oih = GetObjectReader().ReadObjectInfoHeaderFromOid(oid, true);
			}
			// Only necessary to check if there is some trigger
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = GetMetaModel().GetClassInfoFromId
				(oih.GetClassInfoId());
			string className = ci.GetFullClassName();
			bool hasTriggers = triggerManager.HasDeleteTriggersFor(className);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = null;
			if (hasTriggers)
			{
				nnoi = GetObjectReader().ReadNonNativeObjectInfoFromOid(ci, oid, true, false);
				triggerManager.ManageInsertTriggerBefore(className, nnoi);
			}
			base.DeleteObjectWithOid(oid);
			if (hasTriggers)
			{
				triggerManager.ManageInsertTriggerAfter(className, nnoi, oid);
			}
		}

		/// <summary>TODO Use a mutex to guarantee unique access to the file at this moment.</summary>
		/// <remarks>
		/// TODO Use a mutex to guarantee unique access to the file at this moment.
		/// This should be change
		/// </remarks>
		/// <param name="query"></param>
		/// <param name="inMemory"></param>
		/// <param name="startIndex"></param>
		/// <param name="endIndex"></param>
		/// <param name="returnOjects"></param>
		/// <returns>The object info list</returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		public override NeoDatis.Odb.Objects<T> GetObjectInfos<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex, bool returnOjects)
		{
			try
			{
				NeoDatis.Odb.Core.Layers.Layer3.IObjectReader reader = GetObjectReader();
				NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction queryResultAction = new NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionQueryResultAction<T>
					(query, inMemory, this, returnOjects, reader.GetInstanceBuilder());
				return reader.GetObjectInfos<T>(query, inMemory, startIndex, endIndex, returnOjects, 
					queryResultAction);
			}
			finally
			{
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo AddClass(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 newClassInfo, bool addDependentClasses)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = GetObjectWriter().AddClass(newClassInfo
				, addDependentClasses);
			NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession lsession = (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
				)GetSession(true);
			lsession.SetClassInfoId(newClassInfo.GetFullClassName(), ci.GetId());
			return ci;
		}

		public override bool IsLocal()
		{
			return false;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 classInfoList)
		{
			return GetObjectWriter().AddClasses(classInfoList);
		}
	}
}
