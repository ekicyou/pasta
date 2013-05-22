namespace NeoDatis.Btree.Impl.Multiplevalue
{
	[System.Serializable]
	public abstract class BTreeNodeMultipleValuesPerKey : NeoDatis.Btree.Impl.AbstractBTreeNode
		, NeoDatis.Btree.IBTreeNodeMultipleValuesPerKey
	{
		public BTreeNodeMultipleValuesPerKey() : base()
		{
		}

		public BTreeNodeMultipleValuesPerKey(NeoDatis.Btree.IBTree btree) : base(btree)
		{
		}

		public virtual System.Collections.IList GetValueAt(int index)
		{
			return (System.Collections.IList)values[index];
		}

		public override void InsertKeyAndValue(System.IComparable key, object value)
		{
			int position = GetPositionOfKey(key);
			bool addToExistingCollection = false;
			int realPosition = 0;
			if (position >= 0)
			{
				addToExistingCollection = true;
				realPosition = position - 1;
			}
			else
			{
				realPosition = -position - 1;
			}
			// If there is an element at this position and the key is different,
			// then right shift, size
			// safety is guaranteed by the rightShiftFrom method
			if (realPosition < nbKeys && key.CompareTo(keys[realPosition]) != 0)
			{
				RightShiftFrom(realPosition, true);
			}
			keys[realPosition] = key;
			// This is a non unique btree node, manage collection
			ManageCollectionValue(realPosition, value);
			if (!addToExistingCollection)
			{
				nbKeys++;
			}
		}

		/// <param name="realPosition"></param>
		/// <param name="value"></param>
		private void ManageCollectionValue(int realPosition, object value)
		{
			object o = values[realPosition];
			if (o == null)
			{
				o = new System.Collections.ArrayList();
				values[realPosition] = o;
			}
			else
			{
				if (!(o is System.Collections.IList))
				{
					throw new NeoDatis.Btree.Exception.BTreeException("Value of Non Unique Value BTree should be collection and it is "
						 + o.GetType().FullName);
				}
			}
			System.Collections.IList l = (System.Collections.IList)o;
			l.Add(value);
		}

		public virtual System.Collections.IList Search(System.IComparable key)
		{
			int position = GetPositionOfKey(key);
			bool keyIsHere = position > 0;
			int realPosition = -1;
			if (keyIsHere)
			{
				realPosition = position - 1;
				System.Collections.IList value = GetValueAt(realPosition);
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
			NeoDatis.Btree.IBTreeNodeMultipleValuesPerKey node = (NeoDatis.Btree.IBTreeNodeMultipleValuesPerKey
				)GetChildAt(realPosition, true);
			return node.Search(key);
		}

		public override object DeleteKeyForLeafNode(NeoDatis.Btree.IKeyAndValue keyAndValue
			)
		{
			bool objectHasBeenFound = false;
			int position = GetPositionOfKey(keyAndValue.GetKey());
			if (position < 0)
			{
				return null;
			}
			int realPosition = position - 1;
			// In Multiple Values per key, the value is a list
			System.Collections.IList value = (System.Collections.IList)values[realPosition];
			// Here we must search for the right object. The list can contains more than 1 object
			int size = value.Count;
			for (int i = 0; i < size && !objectHasBeenFound; i++)
			{
				if (value[i].Equals(keyAndValue.GetValue()))
				{
					value.Remove(i);
					objectHasBeenFound = true;
				}
			}
			if (!objectHasBeenFound)
			{
				return null;
			}
			// If after removal, the list is empty, then remove the key from the node
			if (value.Count == 0)
			{
				// If we get there
				LeftShiftFrom(realPosition, false);
				nbKeys--;
			}
			NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(this);
			return keyAndValue.GetValue();
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
