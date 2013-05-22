namespace NeoDatis.Btree
{
	/// <author>olivier</author>
	public interface IKeyAndValue
	{
		string ToString();

		System.IComparable GetKey();

		void SetKey(System.IComparable key);

		object GetValue();

		void SetValue(object value);
	}
}
