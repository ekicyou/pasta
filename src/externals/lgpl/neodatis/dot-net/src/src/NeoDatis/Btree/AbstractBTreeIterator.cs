namespace NeoDatis.Btree
{
	/// <author>olivier</author>
	public abstract class AbstractBTreeIterator<T> :IterarorAdapter, System.Collections.Generic.IEnumerator<T>
	{
		private NeoDatis.Btree.IBTree btree;

		/// <summary>The current node where the iterator is</summary>
		protected NeoDatis.Btree.IBTreeNode currentNode;

		/// <summary>The current key in the current node where the iterator is</summary>
		private int currentKeyIndex;

		/// <summary>The number of returned keys</summary>
		private int nbReturnedKeys;

		/// <summary>
		/// The number of returned elements ; it may be different from the number of
		/// keys in the case f multileValues btree where a key can contain more than
		/// one value
		/// </summary>
		protected int nbReturnedElements;

		private NeoDatis.Odb.Core.OrderByConstants orderByType;

		public AbstractBTreeIterator(NeoDatis.Btree.IBTree tree, NeoDatis.Odb.Core.OrderByConstants
			 orderByType)
		{
			this.btree = tree;
			this.currentNode = tree.GetRoot();
			this.orderByType = orderByType;
			if (orderByType.IsOrderByDesc())
			{
				this.currentKeyIndex = currentNode.GetNbKeys();
			}
			else
			{
				this.currentKeyIndex = 0;
			}
		}

		public abstract object GetValueAt(NeoDatis.Btree.IBTreeNode node, int currentIndex
			);

		public override bool MoveNext()
		{
			return nbReturnedElements < btree.GetSize();
		}
        public override object GetCurrent()
        {
            return Current;
        }
		public virtual T Current
		{
			get
			{
				if (currentKeyIndex > currentNode.GetNbKeys() || nbReturnedElements >= btree.GetSize
					())
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NoMoreObjectsInCollection
						);
				}
				if (orderByType.IsOrderByDesc())
				{
					return NextDesc();
				}
				return NextAsc();
			}
		}

		protected virtual T NextAsc()
		{
			// Try to go down till a leaf
			while (!currentNode.IsLeaf())
			{
				currentNode = currentNode.GetChildAt(currentKeyIndex, true);
				currentKeyIndex = 0;
			}
			// If leaf has more keys
			if (currentKeyIndex < currentNode.GetNbKeys())
			{
				nbReturnedKeys++;
				nbReturnedElements++;
				object nodeValue = GetValueAt(currentNode, currentKeyIndex);
				currentKeyIndex++;
				return (T)nodeValue;
			}
			NeoDatis.Btree.IBTreeNode child = null;
			// else go up till a node with keys
			while (currentKeyIndex >= currentNode.GetNbKeys())
			{
				child = currentNode;
				currentNode = currentNode.GetParent();
				currentKeyIndex = IndexOfChild(currentNode, child);
			}
			nbReturnedElements++;
			nbReturnedKeys++;
			object value = GetValueAt(currentNode, currentKeyIndex);
			currentKeyIndex++;
			return (T)value;
		}

		protected virtual T NextDesc()
		{
			// Try to go down till a leaf
			while (!currentNode.IsLeaf())
			{
				currentNode = currentNode.GetChildAt(currentKeyIndex, true);
				currentKeyIndex = currentNode.GetNbKeys();
			}
			// If leaf has more keys
			if (currentKeyIndex > 0)
			{
				nbReturnedElements++;
				nbReturnedKeys++;
				currentKeyIndex--;
				object nodeValue = GetValueAt(currentNode, currentKeyIndex);
				return (T)nodeValue;
			}
			NeoDatis.Btree.IBTreeNode child = null;
			// else go up till a node will keys
			while (currentKeyIndex == 0)
			{
				child = currentNode;
				currentNode = currentNode.GetParent();
				currentKeyIndex = IndexOfChild(currentNode, child);
			}
			nbReturnedElements++;
			nbReturnedKeys++;
			currentKeyIndex--;
			object value = GetValueAt(currentNode, currentKeyIndex);
			return (T)value;
		}

		private int IndexOfChild(NeoDatis.Btree.IBTreeNode parent, NeoDatis.Btree.IBTreeNode
			 child)
		{
			for (int i = 0; i < parent.GetNbChildren(); i++)
			{
				if (parent.GetChildAt(i, true).GetId() == child.GetId())
				{
					return i;
				}
			}
			throw new System.Exception("parent " + parent + " does not have the specified child : "
				 + child);
		}

		public virtual void Remove()
		{
		}

		// TODO Auto-generated method stub
		public override void Reset()
		{
			this.currentNode = btree.GetRoot();
			if (orderByType.IsOrderByDesc())
			{
				this.currentKeyIndex = currentNode.GetNbKeys();
			}
			else
			{
				this.currentKeyIndex = 0;
			}
			nbReturnedElements = 0;
			nbReturnedKeys = 0;
		}

        public virtual void Dispose(){

        }
	}

}
