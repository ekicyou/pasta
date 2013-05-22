namespace NeoDatis.Btree.Impl.Multiplevalue
{
	[System.Serializable]
	public abstract class BTreeMultipleValuesPerKey : NeoDatis.Btree.Impl.AbstractBTree
		, NeoDatis.Btree.IBTreeMultipleValuesPerKey
	{
		public BTreeMultipleValuesPerKey() : base()
		{
		}

		public BTreeMultipleValuesPerKey(string name, int degree, NeoDatis.Btree.IBTreePersister
			 persister) : base(name, degree, persister)
		{
		}

		public virtual System.Collections.IList Search(System.IComparable key)
		{
			NeoDatis.Btree.IBTreeNodeMultipleValuesPerKey theRoot = (NeoDatis.Btree.IBTreeNodeMultipleValuesPerKey
				)GetRoot();
			return theRoot.Search(key);
		}

		public override System.Collections.IEnumerator Iterator<T>(NeoDatis.Odb.Core.OrderByConstants
			 orderBy)
		{
			return new NeoDatis.Btree.BTreeIteratorMultipleValuesPerKey<T>(this, orderBy);
		}

        public abstract override object GetId();

        public abstract override void SetId(object arg1);
	}
}
