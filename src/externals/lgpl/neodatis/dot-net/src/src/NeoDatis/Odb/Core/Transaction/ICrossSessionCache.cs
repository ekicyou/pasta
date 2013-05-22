namespace NeoDatis.Odb.Core.Transaction
{
	/// <summary>
	/// This interface define the control over objects alive across different
	/// sessions.
	/// </summary>
	/// <remarks>
	/// This interface define the control over objects alive across different
	/// sessions. It is a wrapper for all objects put into cross cache. It's primary
	/// purpose is to maintain references to the linked objects that has been used.
	/// Different strategies can be used on the implementations to support the idea
	/// of a cache based on weak reference.
	/// </remarks>
	/// <since>1.9</since>
	public interface ICrossSessionCache
	{
		/// <summary>
		/// Associates the specified
		/// <see cref="NeoDatis.Odb.OID">NeoDatis.Odb.OID</see>
		/// with the specified object(key) in
		/// this cache
		/// </summary>
		/// <param name="@object">The key. This parameter can not be <code> null </code></param>
		void AddObject(object @object, NeoDatis.Odb.OID oid);

		/// <summary>Removes the mapping for this object from this cache if it is present.</summary>
		/// <remarks>Removes the mapping for this object from this cache if it is present.</remarks>
		/// <param name="@object">
		/// that contains the reference to
		/// <see cref="NeoDatis.Odb.OID">NeoDatis.Odb.OID</see>
		/// . This parameter can not be <code> null </code>
		/// </param>
		void RemoveObject(object @object);

		/// <summary>Mark the object with the oid as deleted.</summary>
		/// <remarks>Mark the object with the oid as deleted.</remarks>
		/// <param name="oid">
		/// that must be marked as deleted.
		/// <pre>
		/// When objects are deleted by oid, the cost is too high to search the object by the oid, so we just keep the deleted oid,
		/// and when looking for an object, check if the oid if is the deleted oids
		/// </pre>
		/// </param>
		void RemoveOid(NeoDatis.Odb.OID oid);

		/// <summary>Returns true if this cache maps one key to the specified object.</summary>
		/// <remarks>Returns true if this cache maps one key to the specified object.</remarks>
		/// <param name="@object"></param>
		/// <returns>boolean</returns>
		bool ExistObject(object @object);

		/// <summary>
		/// Return the specific
		/// <see cref="NeoDatis.Odb.OID">NeoDatis.Odb.OID</see>
		/// </summary>
		/// <param name="@object">
		/// The key on the cache for a
		/// <see cref="NeoDatis.Odb.OID">NeoDatis.Odb.OID</see>
		/// . This parameter can not be <code> null </code>
		/// </param>
		/// <returns>
		/// 
		/// <see cref="NeoDatis.Odb.OID">NeoDatis.Odb.OID</see>
		/// . Returns <code> null </code> in case no find key.
		/// </returns>
		NeoDatis.Odb.OID GetOid(object @object);

		/// <summary>Returns true if this map contains no key-value mappings.</summary>
		/// <remarks>Returns true if this map contains no key-value mappings.</remarks>
		/// <returns>boolean</returns>
		bool IsEmpty();

		/// <summary>Removes all mappings from this cache.</summary>
		/// <remarks>Removes all mappings from this cache.</remarks>
		void Clear();

		/// <summary>Returns a String writing down the objects</summary>
		/// <returns>String</returns>
		string ToString();

		/// <summary>Returns the number of key-value mappings in this cache.</summary>
		/// <remarks>Returns the number of key-value mappings in this cache.</remarks>
		/// <returns>int The amount of objects on the cache</returns>
		int Size();
	}
}
