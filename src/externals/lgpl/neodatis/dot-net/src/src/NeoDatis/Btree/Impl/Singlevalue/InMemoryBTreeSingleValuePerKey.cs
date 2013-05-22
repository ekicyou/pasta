namespace NeoDatis.Btree.Impl.Singlevalue
{
	[System.Serializable]
	public class InMemoryBTreeSingleValuePerKey : NeoDatis.Btree.Impl.AbstractBTree, 
		NeoDatis.Btree.IBTreeSingleValuePerKey
	{
		protected static int nextId = 1;

		protected int id;

		public InMemoryBTreeSingleValuePerKey() : base()
		{
		}

		public InMemoryBTreeSingleValuePerKey(string name, int degree, NeoDatis.Btree.IBTreePersister
			 persister) : base(name, degree, persister)
		{
		}

		public virtual object Search(System.IComparable key)
		{
			NeoDatis.Btree.IBTreeNodeOneValuePerKey theRoot = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)GetRoot();
			return theRoot.Search(key);
		}

		public InMemoryBTreeSingleValuePerKey(string name, int degree) : base(name, degree
			, new NeoDatis.Btree.Impl.InMemoryPersister())
		{
			this.id = nextId++;
		}

		public override NeoDatis.Btree.IBTreeNode BuildNode()
		{
			return new NeoDatis.Btree.Impl.Singlevalue.InMemoryBTreeNodeSingleValuePerkey(this
				);
		}

		public override object GetId()
		{
			return id;
		}

		public override void SetId(object id)
		{
			this.id = (int)id;
		}

		public override void Clear()
		{
		}

		public override System.Collections.IEnumerator Iterator<T>(NeoDatis.Odb.Core.OrderByConstants
			 orderBy)
		{
			return new NeoDatis.Btree.BTreeIteratorSingleValuePerKey<T>(this, orderBy);
		}
	}
}
