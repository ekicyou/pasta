namespace NeoDatis.Odb.Core.Transaction
{
	public interface ISession
	{
		NeoDatis.Odb.Core.Transaction.ICache GetCache();

		NeoDatis.Odb.Core.Transaction.ITmpCache GetTmpCache();

		void Rollback();

		void Close();

		void ClearCache();

		bool IsRollbacked();

		void Clear();

		NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine();

		bool TransactionIsPending();

		void Commit();

		NeoDatis.Odb.Core.Transaction.ITransaction GetTransaction();

		void SetFileSystemInterfaceToApplyTransaction(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsi);

		string GetBaseIdentification();

		NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel GetMetaModel();

		void SetMetaModel(NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel2);

		string GetId();

		void SetId(string id);

		void RemoveObjectFromCache(object @object);

		/// <summary>Add these information on a session cache.</summary>
		/// <remarks>Add these information on a session cache.</remarks>
		void AddObjectToCache(NeoDatis.Odb.OID oid, object @object, NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 oih);
	}
}
