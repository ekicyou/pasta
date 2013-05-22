using System;
using NeoDatis.Btree;
using NeoDatis.Btree.Impl.Singlevalue;
using NeoDatis.Odb.Impl.Core.Btree;
using NUnit.Framework;
namespace NeoDatis.BTree
{
	
	public class MockBTreeNodeFactory
	{
		public static IBTreeNode getBTreeNode(IBTree btree)
		{
            return new ODBBTreeNodeSingle(btree);
		}
	}
}