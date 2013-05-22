namespace NeoDatis.Odb.Core.Server.Connection
{
	/// <author>olivier</author>
	public interface IConnection
	{
		string GetId();

		NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine();

		/// <exception cref="System.Exception"></exception>
		void Close();

		/// <exception cref="System.Exception"></exception>
		void Commit();

		/// <exception cref="System.Exception"></exception>
		void UnlockObjectWithOid(NeoDatis.Odb.OID oid);

		/// <exception cref="System.Exception"></exception>
		void Rollback();

		/// <exception cref="System.Exception"></exception>
		bool LockObjectWithOid(NeoDatis.Odb.OID oid);

		void SetCurrentAction(int action);

		void EndCurrentAction();

		string GetDescription();
	}
}
