namespace NeoDatis.Btree.Impl
{
	[System.Serializable]
	public abstract class AbstractBTreeNode : NeoDatis.Btree.IBTreeNode
	{
		protected int degree;

		protected System.IComparable[] keys;

		protected object[] values;

		protected int nbKeys;

		protected int nbChildren;

		protected int maxNbKeys;

		protected int maxNbChildren;

		/// <summary>The BTree owner of this node</summary>
		[System.NonSerialized]
		protected NeoDatis.Btree.IBTree btree;

		public AbstractBTreeNode()
		{
			this.btree = null;
			this.degree = -1;
			this.maxNbKeys = -1;
			this.maxNbChildren = -1;
			keys = null;
			values = null;
			nbKeys = 0;
			nbChildren = 0;
		}

		public AbstractBTreeNode(NeoDatis.Btree.IBTree btree)
		{
			BasicInit(btree);
		}

		private void BasicInit(NeoDatis.Btree.IBTree btree)
		{
			this.btree = btree;
			this.degree = btree.GetDegree();
			this.maxNbKeys = 2 * degree - 1;
			this.maxNbChildren = 2 * degree;
			keys = new System.IComparable[maxNbKeys];
			values = new object[maxNbKeys];
			nbKeys = 0;
			nbChildren = 0;
			Init();
		}

		public abstract void InsertKeyAndValue(System.IComparable key, object value);

		protected abstract void Init();

		public abstract NeoDatis.Btree.IBTreeNode GetChildAt(int index, bool throwExceptionIfNotExist
			);

		public abstract NeoDatis.Btree.IBTreeNode GetParent();

		public abstract object GetParentId();

		public abstract void SetParent(NeoDatis.Btree.IBTreeNode node);

		public abstract bool HasParent();

		public abstract void MoveChildFromTo(int sourceIndex, int destinationIndex, bool 
			throwExceptionIfDoesNotExist);

		/// <summary>Creates a new node with the right part of this node.</summary>
		/// <remarks>
		/// Creates a new node with the right part of this node. This should only be
		/// called on a full node
		/// </remarks>
		public virtual NeoDatis.Btree.IBTreeNode ExtractRightPart()
		{
			if (!IsFull())
			{
				throw new NeoDatis.Btree.Exception.BTreeException("extract right part called on non full node"
					);
			}
			// Creates an empty new node
			NeoDatis.Btree.IBTreeNode rightPart = btree.BuildNode();
			int j = 0;
			for (int i = degree; i < maxNbKeys; i++)
			{
				rightPart.SetKeyAndValueAt(keys[i], values[i], j, false, false);
				keys[i] = null;
				values[i] = null;
				rightPart.SetChildAt(this, i, j, false);
				// TODO must we load all nodes to set new parent
				NeoDatis.Btree.IBTreeNode c = rightPart.GetChildAt(j, false);
				if (c != null)
				{
					c.SetParent(rightPart);
				}
				// rightPart.setChildAt(getChildAt(i,false), j);
				SetNullChildAt(i);
				j++;
			}
			// rightPart.setChildAt(getLastPositionChild(), j);
			rightPart.SetChildAt(this, GetMaxNbChildren() - 1, j, false);
			// correct father id
			NeoDatis.Btree.IBTreeNode c1 = rightPart.GetChildAt(j, false);
			if (c1 != null)
			{
				c1.SetParent(rightPart);
			}
			SetNullChildAt(maxNbChildren - 1);
			// resets last child
			keys[degree - 1] = null;
			// resets median element
			values[degree - 1] = null;
			// set numbers
			nbKeys = degree - 1;
			int originalNbChildren = nbChildren;
			nbChildren = System.Math.Min(nbChildren, degree);
			rightPart.SetNbKeys(degree - 1);
			rightPart.SetNbChildren(originalNbChildren - nbChildren);
			NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(this);
			NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(rightPart);
			NeoDatis.Btree.Tool.BTreeValidator.CheckDuplicateChildren(this, rightPart);
			return rightPart;
		}

		public virtual NeoDatis.Btree.IKeyAndValue GetKeyAndValueAt(int index)
		{
			if (keys[index] == null && values[index] == null)
			{
				return null;
			}
			return new NeoDatis.Btree.Impl.KeyAndValue(keys[index], values[index]);
		}

		public virtual NeoDatis.Btree.IKeyAndValue GetLastKeyAndValue()
		{
			return GetKeyAndValueAt(nbKeys - 1);
		}

		public virtual System.IComparable GetKeyAt(int index)
		{
			return keys[index];
		}

		public virtual NeoDatis.Btree.IKeyAndValue GetMedian()
		{
			int medianPosition = degree - 1;
			return GetKeyAndValueAt(medianPosition);
		}

		/// <summary>Returns the position of the key.</summary>
		/// <remarks>
		/// Returns the position of the key. If the key does not exist in node,
		/// returns the position where this key should be,multiplied by -1
		/// <pre>
		/// for example for node of degree 3 : [1 89 452 789 - ],
		/// calling getPositionOfKey(89) returns 2 (starts with 1)
		/// calling getPositionOfKey(99) returns -2 (starts with 1),because the position should be done, but it does not exist so multiply by -1
		/// this is used to know the child we should descend to!in this case the getChild(2).
		/// </pre>
		/// </remarks>
		/// <param name="key"></param>
		/// <returns>
		/// The position of the key,as a negative number if key does not
		/// exist, warning, the position starts with 1and not 0!
		/// </returns>
		public virtual int GetPositionOfKey(System.IComparable key)
		{
			int i = 0;
			while (i < nbKeys)
			{
				int result = keys[i].CompareTo(key);
				if (result == 0)
				{
					return i + 1;
				}
				if (result > 0)
				{
					return -(i + 1);
				}
				i++;
			}
			return -(i + 1);
		}

		public virtual void IncrementNbChildren()
		{
			nbChildren++;
		}

		public virtual void IncrementNbKeys()
		{
			nbKeys++;
		}

		protected virtual void RightShiftFrom(int position, bool shiftChildren)
		{
			if (IsFull())
			{
				throw new NeoDatis.Btree.Exception.BTreeException("Node is full, can't right shift!"
					);
			}
			// Shift keys
			for (int i = nbKeys; i > position; i--)
			{
				keys[i] = keys[i - 1];
				values[i] = values[i - 1];
			}
			keys[position] = null;
			values[position] = null;
			// Shift children
			if (shiftChildren)
			{
				for (int i = nbChildren; i > position; i--)
				{
					MoveChildFromTo(i - 1, i, true);
				}
				// setChildAt(getChildAt(i-1,true),i);
				SetNullChildAt(position);
			}
		}

		protected virtual void LeftShiftFrom(int position, bool shiftChildren)
		{
			for (int i = position; i < nbKeys - 1; i++)
			{
				keys[i] = keys[i + 1];
				values[i] = values[i + 1];
				if (shiftChildren)
				{
					MoveChildFromTo(i + 1, i, false);
				}
			}
			// setChildAt(getChildAt(i+1,false), i);
			keys[nbKeys - 1] = null;
			values[nbKeys - 1] = null;
			if (shiftChildren)
			{
				MoveChildFromTo(nbKeys, nbKeys - 1, false);
				// setChildAt(getChildAt(nbKeys,false), nbKeys-1);
				SetNullChildAt(nbKeys);
			}
		}

		public virtual void SetKeyAndValueAt(System.IComparable key, object value, int index
			)
		{
			keys[index] = key;
			values[index] = value;
		}

		public virtual void SetKeyAndValueAt(NeoDatis.Btree.IKeyAndValue keyAndValue, int
			 index)
		{
			SetKeyAndValueAt(keyAndValue.GetKey(), keyAndValue.GetValue(), index);
		}

		public virtual void SetKeyAndValueAt(System.IComparable key, object value, int index
			, bool shiftIfAlreadyExist, bool incrementNbKeys)
		{
			if (shiftIfAlreadyExist && index < nbKeys)
			{
				RightShiftFrom(index, true);
			}
			keys[index] = key;
			values[index] = value;
			if (incrementNbKeys)
			{
				nbKeys++;
			}
		}

		public virtual void SetKeyAndValueAt(NeoDatis.Btree.IKeyAndValue keyAndValue, int
			 index, bool shiftIfAlreadyExist, bool incrementNbKeys)
		{
			SetKeyAndValueAt(keyAndValue.GetKey(), keyAndValue.GetValue(), index, shiftIfAlreadyExist
				, incrementNbKeys);
		}

		public virtual bool IsFull()
		{
			return nbKeys == maxNbKeys;
		}

		public virtual bool IsLeaf()
		{
			return nbChildren == 0;
		}

		/// <summary>
		/// Can only merge node without intersection =&gt; the greater key of this must
		/// be smaller than the smallest key of the node
		/// </summary>
		public virtual void MergeWith(NeoDatis.Btree.IBTreeNode node)
		{
			NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(this);
			NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(node);
			CheckIfCanMergeWith(node);
			int j = nbKeys;
			for (int i = 0; i < node.GetNbKeys(); i++)
			{
				SetKeyAndValueAt(node.GetKeyAt(i), node.GetValueAsObjectAt(i), j, false, false);
				SetChildAt(node, i, j, false);
				j++;
			}
			// in this, we have to take the last child
			if (node.GetNbChildren() > node.GetNbKeys())
			{
				SetChildAt(node, node.GetNbChildren() - 1, j, true);
			}
			nbKeys += node.GetNbKeys();
			nbChildren += node.GetNbChildren();
			NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(this);
		}

		private void CheckIfCanMergeWith(NeoDatis.Btree.IBTreeNode node)
		{
			if (nbKeys + node.GetNbKeys() > maxNbKeys)
			{
				throw new NeoDatis.Btree.Exception.BTreeException("Trying to merge two nodes with too many keys "
					 + nbKeys + " + " + node.GetNbKeys() + " > " + maxNbKeys);
			}
			if (nbKeys > 0)
			{
				System.IComparable greatestOfThis = keys[nbKeys - 1];
				System.IComparable smallestOfOther = node.GetKeyAt(0);
				if (greatestOfThis.CompareTo(smallestOfOther) >= 0)
				{
					throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Trying to merge two nodes that have intersections :  "
						 + ToString() + " / " + node);
				}
			}
			if (nbKeys < nbChildren)
			{
				throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Trying to merge two nodes where the first one has more children than keys"
					);
			}
		}

		public virtual void RemoveKeyAndValueAt(int index)
		{
			throw new NeoDatis.Btree.Exception.BTreeException("Not implemented");
		}

		public virtual NeoDatis.Btree.IBTreeNode GetLastChild()
		{
			return GetChildAt(nbChildren - 1, true);
		}

		public virtual NeoDatis.Btree.IBTreeNode GetLastPositionChild()
		{
			return GetChildAt(maxNbChildren - 1, false);
		}

		public virtual int GetNbKeys()
		{
			return nbKeys;
		}

		public virtual void SetNbChildren(int nbChildren)
		{
			this.nbChildren = nbChildren;
		}

		public virtual void SetNbKeys(int nbKeys)
		{
			this.nbKeys = nbKeys;
		}

		public virtual int GetDegree()
		{
			return degree;
		}

		public virtual int GetNbChildren()
		{
			return nbChildren;
		}

		public virtual object DeleteKeyForLeafNode(NeoDatis.Btree.IKeyAndValue keyAndValue
			)
		{
			int position = GetPositionOfKey(keyAndValue.GetKey());
			if (position < 0)
			{
				return null;
			}
			int realPosition = position - 1;
			object value = values[realPosition];
			LeftShiftFrom(realPosition, false);
			nbKeys--;
			NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(this);
			return value;
		}

		public virtual object DeleteKeyAndValueAt(int keyIndex, bool shiftChildren)
		{
			object currentValue = values[keyIndex];
			LeftShiftFrom(keyIndex, shiftChildren);
			nbKeys--;
			// only decrease child number if child are involved in shifting
			if (shiftChildren && nbChildren > keyIndex)
			{
				nbChildren--;
			}
			// BTreeValidator.validateNode(this);
			return currentValue;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("id=").Append(GetId()).Append(" {keys(").Append(nbKeys).Append(")=("
				);
			for (int i = 0; i < nbKeys; i++)
			{
				if (i > 0)
				{
					buffer.Append(",");
				}
				buffer.Append(keys[i]).Append("/").Append(values[i]);
			}
			buffer.Append("), child(").Append(nbChildren).Append(")}");
			return buffer.ToString();
		}

		public virtual int GetMaxNbChildren()
		{
			return maxNbChildren;
		}

		public virtual void SetBTree(NeoDatis.Btree.IBTree btree)
		{
			this.btree = btree;
		}

		public virtual NeoDatis.Btree.IBTree GetBTree()
		{
			return btree;
		}

		public virtual void Clear()
		{
			BasicInit(btree);
		}

		public abstract void DeleteChildAt(int arg1);

		public abstract object GetChildIdAt(int arg1, bool arg2);

		public abstract object GetId();

		public abstract object GetValueAsObjectAt(int arg1);

		public abstract void SetChildAt(NeoDatis.Btree.IBTreeNode arg1, int arg2, int arg3
			, bool arg4);

		public abstract void SetChildAt(NeoDatis.Btree.IBTreeNode arg1, int arg2);

		public abstract void SetId(object arg1);

		public abstract void SetNullChildAt(int arg1);
	}
}
