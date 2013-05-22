namespace NeoDatis.Btree
{
	/// <summary>An iterator to iterate over NeoDatis BTree.</summary>
	/// <remarks>An iterator to iterate over NeoDatis BTree.</remarks>
	/// <author>olivier</author>
	public class BTreeIteratorSingleValuePerKey<T> : NeoDatis.Btree.AbstractBTreeIterator<T>
	{
		public BTreeIteratorSingleValuePerKey(NeoDatis.Btree.IBTree tree, NeoDatis.Odb.Core.OrderByConstants
			 orderByType) : base(tree, orderByType)
		{
		}

		public override object GetValueAt(NeoDatis.Btree.IBTreeNode node, int currentIndex
			)
		{
			NeoDatis.Btree.IBTreeNodeOneValuePerKey n = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)node;
			return n.GetValueAt(currentIndex);
		}
	}
}
