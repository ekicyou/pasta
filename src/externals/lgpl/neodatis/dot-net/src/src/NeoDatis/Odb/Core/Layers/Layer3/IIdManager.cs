namespace NeoDatis.Odb.Core.Layers.Layer3
{
	public interface IIdManager
	{
		/// <summary>Gets an id for an object (instance)</summary>
		/// <param name="objectPosition">the object position (instance)</param>
		/// <returns>The id</returns>
		NeoDatis.Odb.OID GetNextObjectId(long objectPosition);

		/// <summary>Gets an id for a class</summary>
		/// <param name="objectPosition">the object position (class)</param>
		/// <returns>The id</returns>
		NeoDatis.Odb.OID GetNextClassId(long objectPosition);

		void UpdateObjectPositionForOid(NeoDatis.Odb.OID oid, long objectPosition, bool writeInTransaction
			);

		void UpdateClassPositionForId(NeoDatis.Odb.OID classId, long objectPosition, bool
			 writeInTransaction);

		void UpdateIdStatus(NeoDatis.Odb.OID id, byte newStatus);

		void ReserveIds(long nbIds);

		long GetObjectPositionWithOid(NeoDatis.Odb.OID oid, bool useCache);

		void Clear();

		/// <summary>To check if the id block must shift: that a new id block must be created
		/// 	</summary>
		/// <returns>a boolean value to check if block of id is full</returns>
		bool MustShift();

		NeoDatis.Odb.OID ConsultNextOid();
	}
}
