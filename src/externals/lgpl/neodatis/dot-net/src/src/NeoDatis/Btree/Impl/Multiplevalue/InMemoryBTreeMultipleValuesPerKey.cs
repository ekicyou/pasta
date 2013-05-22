namespace NeoDatis.Btree.Impl.Multiplevalue
{
	[System.Serializable]
	public class InMemoryBTreeMultipleValuesPerKey : NeoDatis.Btree.Impl.AbstractBTree
		, NeoDatis.Btree.IBTreeMultipleValuesPerKey
	{
		protected static int nextId = 1;

		protected int id;

		public InMemoryBTreeMultipleValuesPerKey() : base()
		{
		}

		public InMemoryBTreeMultipleValuesPerKey(string name, int degree, NeoDatis.Btree.IBTreePersister
			 persister) : base(name, degree, persister)
		{
		}

		public virtual System.Collections.IList Search(System.IComparable key)
		{
			NeoDatis.Btree.IBTreeNodeMultipleValuesPerKey theRoot = (NeoDatis.Btree.IBTreeNodeMultipleValuesPerKey
				)GetRoot();
			return theRoot.Search(key);
		}

		public InMemoryBTreeMultipleValuesPerKey(string name, int degree) : base(name, degree
			, new NeoDatis.Btree.Impl.InMemoryPersister())
		{
			this.id = nextId++;
		}

		public override NeoDatis.Btree.IBTreeNode BuildNode()
		{
			return new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeNodeMultipleValuesPerKey
				(this);
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
			return new NeoDatis.Btree.BTreeIteratorMultipleValuesPerKey<T>(this, orderBy);
		}
	}
}
