namespace NeoDatis.Btree
{
	/// <summary>
	/// An iterator to iterate over NeoDatis BTree that accept more than one value
	/// per key.
	/// </summary>
	/// <remarks>
	/// An iterator to iterate over NeoDatis BTree that accept more than one value
	/// per key. This is used for non unique index and collection that return ordered
	/// by results
	/// </remarks>
	/// <author>olivier</author>
	public class BTreeIteratorMultipleValuesPerKey<T> : NeoDatis.Btree.AbstractBTreeIterator<T>
	{
		/// <param name="tree"></param>
		/// <param name="orderByType"></param>
		public BTreeIteratorMultipleValuesPerKey(NeoDatis.Btree.IBTree tree, NeoDatis.Odb.Core.OrderByConstants
			 orderByType) : base(tree, orderByType)
		{
			currenListIndex = 0;
			currentValue = null;
		}

		/// <summary>
		/// The index in the list of the current value, Here values of a key are
		/// lists!
		/// </summary>
		private int currenListIndex;

		/// <summary>The current value(List) of the current key being read.</summary>
		/// <remarks>The current value(List) of the current key being read.</remarks>
		private System.Collections.IList currentValue;

		public override T Current
		{
			get
			{
				// Here , the value of a specific key is a list, so we must iterate
				// through the list before going
				// to the next node
				if (currentNode != null && currentValue != null)
				{
					int listSize = currentValue.Count;
					if (listSize > currenListIndex)
					{
						object value = currentValue[currenListIndex];
						currenListIndex++;
						nbReturnedElements++;
						return (T)value;
					}
					// We have reached the end of the list or the list is empty
					// We must continue iterate in the current node / btree
					currenListIndex = 0;
					currentValue = null;
				}
				return base.Current;
			}
		}

		public override object GetValueAt(NeoDatis.Btree.IBTreeNode node, int currentIndex
			)
		{
			if (currentValue == null)
			{
				currentValue = (System.Collections.IList)node.GetValueAsObjectAt(currentIndex);
			}
			int listSize = currentValue.Count;
			if (listSize > currenListIndex)
			{
				object value = currentValue[currenListIndex];
				currenListIndex++;
				return value;
			}
			// We have reached the end of the list or the list is empty
			// We must continue iterate in the current node / btree
			currenListIndex = 0;
			currentValue = null;
			return null;
		}
	}
}
