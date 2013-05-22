namespace NeoDatis.Odb.Core.Layers.Layer3
{
	/// <summary>The interface of all that a StorageEngine (Main concept in ODB) must do.
	/// 	</summary>
	/// <remarks>The interface of all that a StorageEngine (Main concept in ODB) must do.
	/// 	</remarks>
	/// <author>osmadja</author>
	public interface IStorageEngine
	{
		NeoDatis.Odb.OID Store(NeoDatis.Odb.OID oid, object @object);

		/// <summary>Store an object in an database.</summary>
		/// <remarks>
		/// Store an object in an database.
		/// To detect if object must be updated or insert, we use the cache. To
		/// update an object, it must be first selected from the database. When an
		/// object is to be stored, if it exist in the cache, then it will be
		/// updated, else it will be inserted as a new object. If the object is null,
		/// the cache will be used to check if the meta representation is in the
		/// cache
		/// </remarks>
		NeoDatis.Odb.OID Store(object @object);

		void DeleteObjectWithOid(NeoDatis.Odb.OID oid);

		NeoDatis.Odb.OID Delete(object @object);

		void Close();

		long Count(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query);

		NeoDatis.Odb.Values GetValues(NeoDatis.Odb.Core.Query.IValuesQuery query, int startIndex
			, int endIndex);

		NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery query, bool 
			inMemory, int startIndex, int endIndex);

		NeoDatis.Odb.Objects<T> GetObjects<T>(System.Type clazz, bool inMemory, int startIndex
			, int endIndex);

		/// <summary>Return Meta representation of objects</summary>
		/// <param name="query">The query to select objects</param>
		/// <param name="inMemory">To indicate if object must be all loaded in memory</param>
		/// <param name="startIndex">First object index</param>
		/// <param name="endIndex">Last object index</param>
		/// <param name="returnOjects">To indicate if object instances must be created</param>
		/// <returns>The list of objects @</returns>
		NeoDatis.Odb.Objects<T> GetObjectInfos<T>(NeoDatis.Odb.Core.Query.IQuery query, bool
			 inMemory, int startIndex, int endIndex, bool returnOjects);

		NeoDatis.Odb.Core.Layers.Layer3.IObjectReader GetObjectReader();

		NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter GetObjectWriter();

		NeoDatis.Odb.Core.Trigger.ITriggerManager GetTriggerManager();

		NeoDatis.Odb.Core.Transaction.ISession GetSession(bool throwExceptionIfDoesNotExist
			);

		NeoDatis.Odb.Core.Transaction.ISession BuildDefaultSession();

		void Commit();

		void Rollback();

		NeoDatis.Odb.OID GetObjectId(object @object, bool throwExceptionIfDoesNotExist);

		object GetObjectFromOid(NeoDatis.Odb.OID oid);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetMetaObjectFromOid(NeoDatis.Odb.OID
			 oid);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeaderFromOid(
			NeoDatis.Odb.OID oid);

		void DefragmentTo(string newFileName);

		System.Collections.Generic.IList<long> GetAllObjectIds();

		System.Collections.Generic.IList<NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo
			> GetAllObjectIdInfos(string objectType, bool displayObjects);

		/// <returns>Returns the currentIdBlockNumber.</returns>
		int GetCurrentIdBlockNumber();

		/// <returns>Returns the currentIdBlockPosition.</returns>
		long GetCurrentIdBlockPosition();

		/// <returns>Returns the currentIdBlockMaxId.</returns>
		NeoDatis.Odb.OID GetCurrentIdBlockMaxOid();

		NeoDatis.Odb.OID GetMaxOid();

		bool IsClosed();

		int GetVersion();

		void AddUpdateTriggerFor(string className, NeoDatis.Odb.Core.Trigger.UpdateTrigger
			 trigger);

		void AddInsertTriggerFor(string className, NeoDatis.Odb.Core.Trigger.InsertTrigger
			 trigger);

		void AddDeleteTriggerFor(string className, NeoDatis.Odb.Core.Trigger.DeleteTrigger
			 trigger);

		void AddSelectTriggerFor(string className, NeoDatis.Odb.Core.Trigger.SelectTrigger
			 trigger);

		void SetVersion(int version);

		void SetDatabaseId(NeoDatis.Odb.DatabaseId databaseId);

		void SetNbClasses(long nbClasses);

		void SetLastODBCloseStatus(bool lastCloseStatus);

		void SetCurrentIdBlockInfos(long currentBlockPosition, int currentBlockNumber, NeoDatis.Odb.OID
			 maxId);

		void SetMetaModel(NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel);

		NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification GetBaseIdentification();

		/// <summary>Write an object already transformed into meta representation!</summary>
		/// <param name="oid"></param>
		/// <param name="nnoi"></param>
		/// <param name="position"></param>
		/// <param name="updatePointers"></param>
		/// <returns>te object position(or id (if &lt;0, it is id)) @</returns>
		NeoDatis.Odb.OID WriteObjectInfo(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, long position, bool updatePointers);

		/// <summary>Updates an object already transformed into meta representation!</summary>
		/// <param name="nnoi">The Object Meta representation</param>
		/// <param name="forceUpdate"></param>
		/// <returns>The OID of the update object @</returns>
		NeoDatis.Odb.OID UpdateObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, bool forceUpdate);

		void AddSession(NeoDatis.Odb.Core.Transaction.ISession session, bool readMetamodel
			);

		/// <param name="className">The class name on which the index must be created</param>
		/// <param name="name">The name of the index</param>
		/// <param name="indexFields">The list of fields of the index</param>
		/// <param name="verbose">
		/// A boolean value to indicate of ODB must describe what it is
		/// doing @ @
		/// </param>
		void AddIndexOn(string className, string name, string[] indexFields, bool verbose
			, bool acceptMultipleValuesForSameKey);

		void AddCommitListener(NeoDatis.Odb.Core.Layers.Layer3.ICommitListener commitListener
			);

		NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			> GetCommitListeners();

		/// <summary>Returns the object used to refactor the database</summary>
		NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager GetRefactorManager();

		void ResetCommitListeners();

		/// <summary>
		/// Used to know if the storage engine is executed in local mode (embedded
		/// mode) or client server mode
		/// </summary>
		bool IsLocal();

		NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 classInfoList);

		NeoDatis.Odb.DatabaseId GetDatabaseId();

		NeoDatis.Odb.TransactionId GetCurrentTransactionId();

		void SetCurrentTransactionId(NeoDatis.Odb.TransactionId transactionId);

		/// <summary>Used to reconnect an object to the current session</summary>
		void Reconnect(object @object);

		/// <summary>Used to disconnect the object from the current session.</summary>
		/// <remarks>
		/// Used to disconnect the object from the current session. The object is
		/// removed from the cache
		/// </remarks>
		void Disconnect(object @object);

		/// <param name="className"></param>
		/// <param name="indexName"></param>
		/// <param name="verbose"></param>
		void RebuildIndex(string className, string indexName, bool verbose);

		/// <param name="className"></param>
		/// <param name="indexName"></param>
		/// <param name="verbose"></param>
		void DeleteIndex(string className, string indexName, bool verbose);

		/// <summary>
		/// Receive the current class info (loaded from current java classes present on classpath
		/// and check against the persisted meta model
		/// </summary>
		/// <param name="currentCIs"></param>
		NeoDatis.Odb.Core.Layers.Layer3.Engine.CheckMetaModelResult CheckMetaModelCompatibility
			(System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> currentCIs);

		NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector BuildObjectIntrospector
			();

		NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter BuildObjectWriter();

		NeoDatis.Odb.Core.Layers.Layer3.IObjectReader BuildObjectReader();

		NeoDatis.Odb.Core.Trigger.ITriggerManager BuildTriggerManager();

		NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector GetObjectIntrospector
			();

		/// <param name="clazz"></param>
		/// <param name="criterion"></param>
		/// <returns></returns>
		NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery(System.Type clazz
			, NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion);

		/// <param name="clazz"></param>
		/// <returns></returns>
		NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery(System.Type clazz
			);
	}
}
