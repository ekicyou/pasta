namespace NeoDatis.Odb.Core.Layers.Layer3
{
	public interface IObjectReader
	{
		/// <summary>Reads the database header</summary>
		/// <param name="user"></param>
		/// <param name="password"></param>
		void ReadDatabaseHeader(string user, string password);

		/// <summary>Reads the database meta model</summary>
		/// <param name="metaModel">An empty meta model</param>
		/// <param name="full">To indicate if a full read must be done</param>
		/// <returns>The modified metamodel</returns>
		NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel ReadMetaModel(NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel
			 metaModel, bool full);

		NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex
			> ReadClassInfoIndexesAt(long position, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 classInfo);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo ReadNonNativeObjectInfoFromOid
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, NeoDatis.Odb.OID oid, 
			bool useCache, bool returnObjects);

		/// <summary>reads some attributes of an object.</summary>
		/// <remarks>
		/// reads some attributes of an object.
		/// <pre>
		/// example of method call
		/// readObjectInfoValues(classinfo,18000,true,[&quot;profile.name&quot;,&quot;profile.email&quot;],[&quot;profile.name&quot;,&quot;profile.email&quot;],0)
		/// readObjectInfoValues(classinfo,21789,true,[&quot;name&quot;],[&quot;profile.name&quot;],1)
		/// </pre>
		/// </remarks>
		/// <param name="classInfo">
		/// If null, we are probably reading a native instance : String
		/// for example
		/// </param>
		/// <param name="oid">
		/// The oid of the object to read. if -1,the read will be done by
		/// position
		/// </param>
		/// <param name="useCache">
		/// To indicate if cache must be used. If not, the old version of
		/// the object will read
		/// </param>
		/// <param name="attributeNames">
		/// The names of attributes to read the values, an attributename
		/// can contain relation like profile.name
		/// </param>
		/// <param name="relationAttributeNames">
		/// The original names of attributes to read the values, an
		/// attributename can contain relation like profile.name
		/// </param>
		/// <param name="recursionLevel">The recursion level of this method call</param>
		/// <returns>The map with attribute values</returns>
		NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap ReadObjectInfoValuesFromOID
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, NeoDatis.Odb.OID oid, 
			bool useCache, NeoDatis.Tool.Wrappers.List.IOdbList<string> attributeNames, NeoDatis.Tool.Wrappers.List.IOdbList
			<string> relationAttributeNames, int recursionLevel, string[] orderByFields);

		object ReadAtomicNativeObjectInfoAsObject(long position, int odbTypeId);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo ReadAtomicNativeObjectInfo
			(long position, int odbTypeId);

		long ReadOidPosition(NeoDatis.Odb.OID oid);

		object GetObjectFromOid(NeoDatis.Odb.OID oid, bool returnInstance, bool useCache);

		System.Collections.Generic.IList<NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo
			> GetAllIdInfos(string objectTypeToDisplay, byte idType, bool displayObject);

		/// <summary>Returns the id of an object by reading the object header</summary>
		/// <param name="position"></param>
		/// <param name="includeDeleted"></param>
		/// <returns>The oid of the object at the specific position</returns>
		NeoDatis.Odb.OID GetIdOfObjectAt(long position, bool includeDeleted);

		void Close();

		object BuildOneInstance(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo 
			objectInfo);

		/// <summary>Get a list of object matching the query</summary>
		/// <param name="query"></param>
		/// <param name="inMemory"></param>
		/// <param name="startIndex"></param>
		/// <param name="endIndex"></param>
		/// <returns>The list of objects</returns>
		NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery query, bool 
			inMemory, int startIndex, int endIndex);

		/// <summary>Get a list of values matching the query</summary>
		/// <param name="query"></param>
		/// <param name="startIndex"></param>
		/// <param name="endIndex"></param>
		/// <returns>The list of values</returns>
		NeoDatis.Odb.Values GetValues(NeoDatis.Odb.Core.Query.IValuesQuery query, int startIndex
			, int endIndex);

		/// <summary>Return Objects.</summary>
		/// <remarks>
		/// Return Objects. Match the query without instantiating objects. Only
		/// instantiate object for object that match the query
		/// </remarks>
		/// <param name="query">The query to select objects</param>
		/// <param name="inMemory">To indicate if object must be all loaded in memory</param>
		/// <param name="startIndex">First object index</param>
		/// <param name="endIndex">Last object index</param>
		/// <param name="returnObjects">To indicate if object instances must be created</param>
		/// <returns>The list of objects</returns>
		NeoDatis.Odb.Objects<T> GetObjectInfos<T>(NeoDatis.Odb.Core.Query.IQuery query, bool
			 inMemory, int startIndex, int endIndex, bool returnObjects, NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction
			 queryResultAction);

		string GetBaseIdentification();

		NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder GetInstanceBuilder();

		/// <summary>Reads the pointers(ids or positions) of an object that has the specific oid
		/// 	</summary>
		/// <param name="oid">The oid of the object we want to read the pointers</param>
		/// <returns>The ObjectInfoHeader</returns>
		NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader ReadObjectInfoHeaderFromOid
			(NeoDatis.Odb.OID oid, bool useCache);

		/// <summary>Returns information about all OIDs of the database</summary>
		/// <param name="idType"></param>
		/// <returns></returns>
		System.Collections.Generic.IList<long> GetAllIds(byte idType);

		/// <summary>Gets the next object oid of the object with the specific oid</summary>
		/// <param name="oid"></param>
		/// <returns>
		/// The oid of the next object. If there is no next object,
		/// return null
		/// </returns>
		NeoDatis.Odb.OID GetNextObjectOID(NeoDatis.Odb.OID oid);

		/// <summary>Gets the real object position from its OID</summary>
		/// <param name="oid">
		/// The oid of the object to get the position
		/// To indicate if an exception must be thrown if object is not
		/// found
		/// </param>
		/// <returns>
		/// The object position, if object has been marked as deleted then
		/// return StorageEngineConstant.DELETED_OBJECT_POSITION
		/// </returns>
		long GetObjectPositionFromItsOid(NeoDatis.Odb.OID oid, bool useCache, bool throwException
			);

		/// <summary>Reads a non non native Object Info (Layer2) from its position</summary>
		/// <param name="classInfo"></param>
		/// <param name="oid">can be null</param>
		/// <param name="position"></param>
		/// <param name="useCache"></param>
		/// <param name="returnInstance"></param>
		/// <returns>The meta representation of the object</returns>
		NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo ReadNonNativeObjectInfoFromPosition
			(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, NeoDatis.Odb.OID oid, 
			long position, bool useCache, bool returnInstance);
	}
}
