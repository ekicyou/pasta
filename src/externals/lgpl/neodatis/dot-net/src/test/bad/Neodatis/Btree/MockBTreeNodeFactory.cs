using System;
using NeoDatis.Btree;
namespace NeoDatis.BTree
{
	
	public class MockBTreeNodeFactory
	{
		public static IBTreeNode getBTreeNode(IBTree btree)
		{
			return new InMemoryBTreeNode(btree);
		}
	}
}