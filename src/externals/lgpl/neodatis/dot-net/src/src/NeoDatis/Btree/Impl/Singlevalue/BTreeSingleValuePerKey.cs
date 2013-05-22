namespace NeoDatis.Btree.Impl.Singlevalue
{
	[System.Serializable]
	public abstract class BTreeSingleValuePerKey : NeoDatis.Btree.Impl.AbstractBTree, 
		NeoDatis.Btree.IBTreeSingleValuePerKey
	{
		public BTreeSingleValuePerKey()
		{
		}

		public BTreeSingleValuePerKey(string name, int degree, NeoDatis.Btree.IBTreePersister
			 persister) : base(name, degree, persister)
		{
		}

		public virtual object Search(System.IComparable key)
		{
			NeoDatis.Btree.IBTreeNodeOneValuePerKey theRoot = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)GetRoot();
			return theRoot.Search(key);
		}

		public override System.Collections.IEnumerator Iterator<T>(NeoDatis.Odb.Core.OrderByConstants
			 orderBy)
		{
			return new NeoDatis.Btree.BTreeIteratorSingleValuePerKey<T>(this, orderBy);
		}

        public abstract override object GetId();

        public abstract override void SetId(object arg1);
	}
}
