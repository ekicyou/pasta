namespace NeoDatis.Btree.Impl.Singlevalue
{
	[System.Serializable]
	public abstract class BTreeNodeSingleValuePerKey : NeoDatis.Btree.Impl.AbstractBTreeNode
		, NeoDatis.Btree.IBTreeNodeOneValuePerKey
	{
		public BTreeNodeSingleValuePerKey() : base()
		{
		}

		public BTreeNodeSingleValuePerKey(NeoDatis.Btree.IBTree btree) : base(btree)
		{
		}

		public virtual object GetValueAt(int index)
		{
			return values[index];
		}

		public override void InsertKeyAndValue(System.IComparable key, object value)
		{
			int position = GetPositionOfKey(key);
			int realPosition = 0;
			if (position >= 0)
			{
				throw new NeoDatis.Btree.Exception.DuplicatedKeyException(key.ToString());
			}
			realPosition = -position - 1;
			// If there is an element at this position, then right shift, size
			// safety is guaranteed by the rightShiftFrom method
			if (realPosition < nbKeys)
			{
				RightShiftFrom(realPosition, true);
			}
			keys[realPosition] = key;
			values[realPosition] = value;
			nbKeys++;
		}

		public virtual object Search(System.IComparable key)
		{
			int position = GetPositionOfKey(key);
			bool keyIsHere = position > 0;
			int realPosition = -1;
			if (keyIsHere)
			{
				realPosition = position - 1;
				object value = GetValueAt(realPosition);
				return value;
			}
			else
			{
				if (IsLeaf())
				{
					// key is not here and node is leaf
					return null;
				}
			}
			realPosition = -position - 1;
			NeoDatis.Btree.IBTreeNodeOneValuePerKey node = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)GetChildAt(realPosition, true);
			return node.Search(key);
		}

		public abstract override void DeleteChildAt(int arg1);

        public abstract override object GetChildIdAt(int arg1, bool arg2);

        public abstract override object GetId();

        public abstract override object GetValueAsObjectAt(int arg1);

        public abstract override void SetChildAt(NeoDatis.Btree.IBTreeNode arg1, int arg2, int arg3
			, bool arg4);

        public abstract override void SetChildAt(NeoDatis.Btree.IBTreeNode arg1, int arg2);

        public abstract override void SetId(object arg1);

        public abstract override void SetNullChildAt(int arg1);
	}
}
