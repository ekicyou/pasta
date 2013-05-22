namespace NeoDatis.Odb
{
	/// <summary>used to give the user an instance of an object representation, level2.</summary>
	/// <remarks>
	/// used to give the user an instance of an object representation, level2. The
	/// Object Representation encapsulates the NonNativeObjectInfo which is the
	/// internal object representation. This is used in the Server triggers.
	/// </remarks>
	/// <author>olivier</author>
	public interface ObjectRepresentation
	{
		/// <summary>Retrieves the oid of the object</summary>
		/// <returns></returns>
		NeoDatis.Odb.OID GetOid();

		/// <summary>Retrieves the full object class name</summary>
		/// <returns></returns>
		string GetObjectClassName();

		/// <summary>Return the value of a specific attribute</summary>
		/// <param name="attributeName"></param>
		/// <returns></returns>
		object GetValueOf(string attributeName);

		/// <summary>Sets the value of a specific attribute</summary>
		/// <param name="attributeName"></param>
		/// <param name="value"></param>
		void SetValueOf(string attributeName, object value);
	}
}
