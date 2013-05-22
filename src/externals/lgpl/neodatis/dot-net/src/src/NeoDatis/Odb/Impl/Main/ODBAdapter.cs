using NeoDatis.Tool;
namespace NeoDatis.Odb.Impl.Main
{
	/// <summary>A basic adapter for ODB interface</summary>
	/// <author>osmadja</author>
	public abstract class ODBAdapter : NeoDatis.Odb.ODB
	{
		protected NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		protected NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector classIntrospector;

		private NeoDatis.Odb.ODBExt ext;

		public ODBAdapter(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine) : 
			base()
		{
			this.storageEngine = storageEngine;
			this.classIntrospector = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassIntrospector
				();
		}

		public virtual void Commit()
		{
			storageEngine.Commit();
		}

		public virtual void Rollback()
		{
			storageEngine.Rollback();
		}

		public virtual void CommitAndClose()
		{
			storageEngine.Commit();
			storageEngine.Close();
		}

		public virtual OID Store(object o)
		{
			return storageEngine.Store(o);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>()
		{
			return storageEngine.GetObjects<T>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(T)), true, -1, -1);
		}

       public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(bool inMemory
			)
		{
			return storageEngine.GetObjects<T>(typeof(T), inMemory, -1, -1);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(bool inMemory
			, int startIndex, int endIndex)
		{
			return storageEngine.GetObjects<T>(typeof(T), inMemory, startIndex, endIndex);
		}

		public virtual void Close()
		{
			storageEngine.Commit();
			storageEngine.Close();
		}

		public virtual NeoDatis.Odb.OID Delete(object @object)
		{
			return storageEngine.Delete(@object);
		}

		/// <summary>Delete an object from the database with the id</summary>
		/// <param name="oid">The object id to be deleted</param>
		/// <></>
		public virtual void DeleteObjectWithId(NeoDatis.Odb.OID oid)
		{
			storageEngine.DeleteObjectWithOid(oid);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery
			 query)
		{
            query.SetFullClassName(typeof(T));
            return storageEngine.GetObjects<T>(query, true, -1, -1);
		}

		public virtual NeoDatis.Odb.Values GetValues(NeoDatis.Odb.Core.Query.IValuesQuery
			 query)
		{
			return storageEngine.GetValues(query, -1, -1);
		}

		public virtual long Count(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
			 query)
		{
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(query).Count("count");
			q.SetPolymorphic(query.IsPolymorphic());
			NeoDatis.Odb.Values values = storageEngine.GetValues(q, -1, -1);
			System.Decimal count = (System.Decimal)values.NextValues().GetByIndex(0);
            return System.Decimal.ToInt64(count);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory)
		{
			return storageEngine.GetObjects<T>(query, inMemory, -1, -1);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex)
		{
            try
            {
                return storageEngine.GetObjects<T>(query, inMemory, startIndex, endIndex);
            }
            catch (ODBRuntimeException e)
            {
                DLogger.Info(e);
                throw e;
            }
		}

		public virtual NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return storageEngine.GetSession(true);
		}

		public virtual NeoDatis.Odb.OID GetObjectId(object @object)
		{
			return storageEngine.GetObjectId(@object, true);
		}

		public virtual object GetObjectFromId(NeoDatis.Odb.OID id)
		{
			return storageEngine.GetObjectFromOid(id);
		}

		public virtual void DefragmentTo(string newFileName)
		{
			storageEngine.DefragmentTo(newFileName);
		}

		public virtual NeoDatis.Odb.ClassRepresentation GetClassRepresentation(System.Type
			 clazz)
		{
			return GetClassRepresentation(clazz.FullName);
		}

		public virtual NeoDatis.Odb.ClassRepresentation GetClassRepresentation(string fullClassName
			)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo = storageEngine.GetSession
				(true).GetMetaModel().GetClassInfo(fullClassName, false);
			if (classInfo == null)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList ciList = classIntrospector.Introspect
					(fullClassName, true);
				storageEngine.AddClasses(ciList);
				classInfo = ciList.GetMainClassInfo();
			}
			return new NeoDatis.Odb.Impl.Main.DefaultClassRepresentation(storageEngine, classInfo
				);
		}

		/// <summary>or shutdown hook</summary>
		public virtual void Run()
		{
			if (!storageEngine.IsClosed())
			{
				NeoDatis.Tool.DLogger.Debug("ODBFactory has not been closed and VM is exiting : force ODBFactory close"
					);
				storageEngine.Close();
			}
		}

		public virtual void AddUpdateTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.UpdateTrigger
			 trigger)
		{
			if (trigger is NeoDatis.Odb.Core.Server.Trigger.ServerUpdateTrigger)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CanNotAssociateServerTriggerToLocalOrClientOdb
					.AddParameter(trigger.GetType().FullName));
			}
			storageEngine.AddUpdateTriggerFor(clazz.FullName, trigger);
		}

		public virtual void AddInsertTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.InsertTrigger
			 trigger)
		{
			if (trigger is NeoDatis.Odb.Core.Server.Trigger.ServerInsertTrigger)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CanNotAssociateServerTriggerToLocalOrClientOdb
					.AddParameter(trigger.GetType().FullName));
			}
			storageEngine.AddInsertTriggerFor(clazz.FullName, trigger);
		}

		public virtual void AddDeleteTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.DeleteTrigger
			 trigger)
		{
			if (trigger is NeoDatis.Odb.Core.Server.Trigger.ServerDeleteTrigger)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CanNotAssociateServerTriggerToLocalOrClientOdb
					.AddParameter(trigger.GetType().FullName));
			}
			storageEngine.AddDeleteTriggerFor(clazz.FullName, trigger);
		}

		public virtual void AddSelectTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.SelectTrigger
			 trigger)
		{
			if (trigger is NeoDatis.Odb.Core.Server.Trigger.ServerSelectTrigger)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CanNotAssociateServerTriggerToLocalOrClientOdb
					.AddParameter(trigger.GetType().FullName));
			}
			storageEngine.AddSelectTriggerFor(clazz.FullName, trigger);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager GetRefactorManager
			()
		{
			return storageEngine.GetRefactorManager();
		}

		public virtual NeoDatis.Odb.ODBExt Ext()
		{
			if (ext == null)
			{
				ext = new NeoDatis.Odb.Impl.Main.ODBExtImpl(storageEngine);
			}
			return ext;
		}

		public virtual void Disconnect(object @object)
		{
			storageEngine.Disconnect(@object);
		}

		public virtual void Reconnect(object @object)
		{
			storageEngine.Reconnect(@object);
		}

		public virtual bool IsClosed()
		{
			return storageEngine.IsClosed();
		}

		public virtual NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery(
			System.Type clazz, NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion)
		{
			return storageEngine.CriteriaQuery(clazz, criterion);
		}

		public virtual NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery(
			System.Type clazz)
		{
			return storageEngine.CriteriaQuery(clazz);
		}

		public virtual string GetName()
		{
			return storageEngine.GetBaseIdentification().GetIdentification();
		}
	}
}
