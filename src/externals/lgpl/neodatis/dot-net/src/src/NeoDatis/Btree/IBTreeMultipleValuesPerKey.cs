namespace NeoDatis.Btree
{
	public interface IBTreeMultipleValuesPerKey : NeoDatis.Btree.IBTree
	{
		object Delete(System.IComparable key, object value);

		System.Collections.IList Search(System.IComparable key);
	}
}
