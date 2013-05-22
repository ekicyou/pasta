namespace NeoDatis.Odb
{
	/// <summary>The main interface of all Object Values query results of NeoDatis ODB</summary>
	/// <author>osmadja</author>
	public interface Values : NeoDatis.Odb.Objects<NeoDatis.Odb.ObjectValues>
	{
		NeoDatis.Odb.ObjectValues NextValues();

		/// <summary>Inform if the internal Iterator has more objects</summary>
		/// <returns></returns>
		bool HasNext();

		/// <summary>Return the first object of the collection, if exist</summary>
		/// <returns></returns>
		NeoDatis.Odb.ObjectValues GetFirst();

		/// <summary>Reset the internal iterator of the collection</summary>
		void Reset();

		/// <summary>Add an object into the collection using a specific ordering key</summary>
		/// <param name="key"></param>
		/// <param name="@object"></param>
		/// <returns></returns>
		bool AddWithKey(NeoDatis.Tool.Wrappers.OdbComparable key, NeoDatis.Odb.ObjectValues
			 @object);

		/// <summary>Add an object into the collection using a specific ordering key</summary>
		/// <param name="key"></param>
		/// <param name="@object"></param>
		/// <returns></returns>
		bool AddWithKey(int key, NeoDatis.Odb.ObjectValues @object);

		/// <summary>
		/// Returns the collection iterator throughout the order by
		/// <see cref="NeoDatis.Odb.Core.OrderByConstants">NeoDatis.Odb.Core.OrderByConstants
		/// 	</see>
		/// </summary>
		/// <param name="orderByType"></param>
		/// <returns></returns>
		System.Collections.Generic.IEnumerator<NeoDatis.Odb.ObjectValues> Iterator(NeoDatis.Odb.Core.OrderByConstants
			 orderByType);
	}
}
