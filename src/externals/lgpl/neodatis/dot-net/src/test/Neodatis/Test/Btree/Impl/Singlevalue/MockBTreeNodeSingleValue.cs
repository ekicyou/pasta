using NUnit.Framework;
namespace NeoDatis.Test.Btree.Impl.Singlevalue
{
	[System.Serializable]
	public class MockBTreeNodeSingleValue : NeoDatis.Btree.Impl.Singlevalue.InMemoryBTreeNodeSingleValuePerkey
	{
		private string name;

		public MockBTreeNodeSingleValue(NeoDatis.Btree.IBTree btree, string name) : base(
			btree)
		{
			this.name = name;
		}
	}
}
