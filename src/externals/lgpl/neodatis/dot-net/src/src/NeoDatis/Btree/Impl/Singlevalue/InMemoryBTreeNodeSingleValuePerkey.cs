namespace NeoDatis.Btree.Impl.Singlevalue
{
	[System.Serializable]
	public class InMemoryBTreeNodeSingleValuePerkey : NeoDatis.Btree.Impl.Singlevalue.BTreeNodeSingleValuePerKey
	{
		protected static int nextId = 1;

		protected int id;

		public InMemoryBTreeNodeSingleValuePerkey(NeoDatis.Btree.IBTree btree) : base(btree
			)
		{
			id = nextId++;
		}

		protected NeoDatis.Btree.IBTreeNode[] children;

		protected NeoDatis.Btree.IBTreeNode parent;

		public override NeoDatis.Btree.IBTreeNode GetChildAt(int index, bool throwExceptionIfNotExist
			)
		{
			if (children[index] == null && throwExceptionIfNotExist)
			{
				throw new NeoDatis.Btree.Exception.BTreeException("Trying to load null child node at index "
					 + index);
			}
			return children[index];
		}

		public override NeoDatis.Btree.IBTreeNode GetParent()
		{
			return parent;
		}

		public override void SetChildAt(NeoDatis.Btree.IBTreeNode child, int index)
		{
			children[index] = child;
			if (child != null)
			{
				child.SetParent(this);
			}
		}

		public override void SetChildAt(NeoDatis.Btree.IBTreeNode node, int childIndex, int
			 index, bool throwExceptionIfDoesNotExist)
		{
			NeoDatis.Btree.IBTreeNode child = node.GetChildAt(childIndex, throwExceptionIfDoesNotExist
				);
			children[index] = child;
			if (child != null)
			{
				child.SetParent(this);
			}
		}

		public override void SetParent(NeoDatis.Btree.IBTreeNode node)
		{
			parent = node;
		}

		public override bool HasParent()
		{
			return parent != null;
		}

		protected override void Init()
		{
			children = new NeoDatis.Btree.IBTreeNode[maxNbChildren];
		}

		public override object GetId()
		{
			return id;
		}

		public override void SetId(object id)
		{
			this.id = (int)id;
		}

		public override void DeleteChildAt(int index)
		{
			children[index] = null;
			nbChildren--;
		}

		public override void MoveChildFromTo(int sourceIndex, int destinationIndex, bool 
			throwExceptionIfDoesNotExist)
		{
			if (children[sourceIndex] == null && throwExceptionIfDoesNotExist)
			{
				throw new NeoDatis.Btree.Exception.BTreeException("Trying to move null child node at index "
					 + sourceIndex);
			}
			children[destinationIndex] = children[sourceIndex];
		}

		public override void SetNullChildAt(int childIndex)
		{
			children[childIndex] = null;
		}

		public override object GetChildIdAt(int childIndex, bool throwExceptionIfDoesNotExist
			)
		{
			if (children[childIndex] == null && throwExceptionIfDoesNotExist)
			{
				throw new NeoDatis.Btree.Exception.BTreeException("Trying to move null child node at index "
					 + childIndex);
			}
			return children[childIndex].GetId();
		}

		public override object GetParentId()
		{
			return id;
		}

		public override object GetValueAsObjectAt(int index)
		{
			return GetValueAt(index);
		}
	}
}
