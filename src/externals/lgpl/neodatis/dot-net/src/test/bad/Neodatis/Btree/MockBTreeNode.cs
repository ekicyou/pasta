using System;
using NeoDatis.Btree;
using NeoDatis.Btree.Impl.Singlevalue;
namespace NeoDatis.BTree
{
	
	public class MockBTreeNode:InMemoryBTreeNodeSingleValuePerkey
	{
		private System.String name;
		
		public MockBTreeNode(IBTree btree, System.String name):base(btree)
		{
			this.name = name;
		}

        public void RightShiftFromTest(int position, bool shiftChildren)
        {
            base.RightShiftFrom(position, shiftChildren);
        }
        public void LeftShiftFromTest(int position, bool shiftChildren)
        {
            base.LeftShiftFrom(position, shiftChildren);
        }
    }
}