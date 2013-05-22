namespace NeoDatis.Odb
{
	/// <summary>An interface to provider extended access to ODB.</summary>
	/// <remarks>An interface to provider extended access to ODB.</remarks>
	/// <author>osmadja</author>
	public interface ODBExt
	{
		/// <summary>Gets the external OID of an Object.</summary>
		/// <remarks>
		/// Gets the external OID of an Object. The external OID contains the ID of
		/// the database + the oid of the object. The External OID can be used to
		/// identify objects outside the ODB database as it should be unique across
		/// databases. It can be used for example to implement a replication process.
		/// </remarks>
		/// <param name="@object"></param>
		/// <returns></returns>
		NeoDatis.Odb.ExternalOID GetObjectExternalOID(object @object);

		/// <summary>Get the Database ID</summary>
		/// <returns></returns>
		NeoDatis.Odb.DatabaseId GetDatabaseId();

		/// <summary>Convert an OID to External OID</summary>
		/// <param name="oid"></param>
		/// <returns>The external OID</returns>
		NeoDatis.Odb.ExternalOID ConvertToExternalOID(NeoDatis.Odb.OID oid);

		/// <summary>Gets the current transaction Id</summary>
		/// <returns>The current transaction Id</returns>
		NeoDatis.Odb.TransactionId GetCurrentTransactionId();

		/// <summary>Returns the object version of the object that has the specified OID</summary>
		int GetObjectVersion(NeoDatis.Odb.OID oid);

		/// <summary>Returns the object creation date in ms since 1/1/1970</summary>
		/// <param name="oid"></param>
		/// <returns>The creation date</returns>
		long GetObjectCreationDate(NeoDatis.Odb.OID oid);

		/// <summary>Returns the object last update date in ms since 1/1/1970</summary>
		/// <param name="oid"></param>
		/// <returns>The last update date</returns>
		long GetObjectUpdateDate(NeoDatis.Odb.OID oid);
	}
}
