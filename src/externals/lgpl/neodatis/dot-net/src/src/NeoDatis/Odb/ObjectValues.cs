namespace NeoDatis.Odb
{
	/// <summary>
	/// Interface that will be implemented to hold a row of a result of an Object
	/// Values Query
	/// </summary>
	/// <author>osmadja</author>
	public interface ObjectValues
	{
		object GetByAlias(string alias);

		object GetByIndex(int index);

		object[] GetValues();
	}
}
