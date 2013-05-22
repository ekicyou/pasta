namespace NeoDatis.Btree
{
	/// <summary>The interface for btree nodes that accept multiple values for each key</summary>
	/// <author>olivier</author>
	public interface IBTreeNodeOneValuePerKey : NeoDatis.Btree.IBTreeNode
	{
		object GetValueAt(int index);

		object Search(System.IComparable key);
	}
}
