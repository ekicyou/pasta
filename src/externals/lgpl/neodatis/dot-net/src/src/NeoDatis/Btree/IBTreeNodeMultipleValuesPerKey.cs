namespace NeoDatis.Btree
{
	/// <summary>The interface for btree nodes that accept One Value Per Key</summary>
	/// <author>olivier</author>
	public interface IBTreeNodeMultipleValuesPerKey : NeoDatis.Btree.IBTreeNode
	{
		System.Collections.IList GetValueAt(int index);

		System.Collections.IList Search(System.IComparable key);
	}
}
