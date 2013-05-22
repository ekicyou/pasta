namespace NeoDatis.Odb
{
	/// <summary>The main interface of all query results of NeoDatis ODB.</summary>
	/// <remarks>
	/// The main interface of all query results of NeoDatis ODB. Objects interface
	/// extends the Collection interface so it provides a standard collection
	/// behavior.
	/// </remarks>
	/// <author>osmadja</author>
	public interface Objects<E> : System.Collections.Generic.ICollection<E>
	{
		/// <summary>Inform if the internal Iterator has more objects</summary>
		/// <returns></returns>
		bool HasNext();

		/// <summary>Returns the next object of the internal iterator of the collection</summary>
		/// <returns></returns>
		E Next();

		/// <summary>Return the first object of the collection, if exist</summary>
		/// <returns></returns>
		E GetFirst();

		/// <summary>Reset the internal iterator of the collection</summary>
		void Reset();

		/// <summary>Add an object into the collection using a specific ordering key</summary>
		/// <param name="key"></param>
		/// <param name="@object">
		/// The object can be an OID, can o NNOI (NonNativeObjectInfo) or
		/// the object
		/// </param>
		/// <returns></returns>
		bool AddWithKey(NeoDatis.Tool.Wrappers.OdbComparable key, E o);

		/// <summary>Add an object into the collection using a specific ordering key</summary>
		/// <param name="key"></param>
		/// <param name="@object"></param>
		/// <returns></returns>
		bool AddWithKey(int key, E o);

		/// <summary>
		/// Returns the collection iterator throughout the order by
		/// <see cref="NeoDatis.Odb.Core.OrderByConstants">NeoDatis.Odb.Core.OrderByConstants
		/// 	</see>
		/// </summary>
		/// <param name="orderByType"></param>
		/// <returns></returns>
		System.Collections.Generic.IEnumerator<E> Iterator(NeoDatis.Odb.Core.OrderByConstants
			 orderByType);

        void AddOid(OID oid);
    }
}
