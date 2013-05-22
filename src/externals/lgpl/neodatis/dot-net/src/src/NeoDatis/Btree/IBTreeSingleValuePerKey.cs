namespace NeoDatis.Btree
{
	public interface IBTreeSingleValuePerKey : NeoDatis.Btree.IBTree
	{
		object Delete(System.IComparable key, object value);

		object Search(System.IComparable key);
	}
}
