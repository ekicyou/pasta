namespace NeoDatis.Odb.Impl.Core.Btree
{
	/// <summary>The NeoDatis ODB BTree Node implementation.</summary>
	/// <remarks>The NeoDatis ODB BTree Node implementation. It extends the DefaultBTreeNode generic implementation to be able to be stored in the ODB database.
	/// 	</remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ODBBTreeNodeMultiple : NeoDatis.Btree.Impl.Multiplevalue.BTreeNodeMultipleValuesPerKey
	{
		protected NeoDatis.Odb.OID oid;

		public ODBBTreeNodeMultiple() : base()
		{
		}

		protected NeoDatis.Odb.OID[] childrenOids;

		protected NeoDatis.Odb.OID parentOid;

		/// <summary>lazy loaded</summary>
		[System.NonSerialized]
		protected NeoDatis.Btree.IBTreeNode parent;

		public ODBBTreeNodeMultiple(NeoDatis.Btree.IBTree btree) : base(btree)
		{
		}

		public override NeoDatis.Btree.IBTreeNode GetChildAt(int index, bool throwExceptionIfNotExist
			)
		{
			NeoDatis.Odb.OID oid = childrenOids[index];
			if (oid == null)
			{
				if (throwExceptionIfNotExist)
				{
					throw new NeoDatis.Btree.Exception.BTreeException("Trying to load null child node at index "
						 + index);
				}
				return null;
			}
			return btree.GetPersister().LoadNodeById(oid);
		}

		public override NeoDatis.Btree.IBTreeNode GetParent()
		{
			if (parent != null)
			{
				return parent;
			}
			parent = btree.GetPersister().LoadNodeById(parentOid);
			return parent;
		}

		public override void SetChildAt(NeoDatis.Btree.IBTreeNode child, int index)
		{
			if (child != null)
			{
				if (child.GetId() == null)
				{
					btree.GetPersister().SaveNode(child);
				}
				childrenOids[index] = (NeoDatis.Odb.OID)child.GetId();
				child.SetParent(this);
			}
			else
			{
				childrenOids[index] = null;
			}
		}

		public override void SetParent(NeoDatis.Btree.IBTreeNode node)
		{
			parent = node;
			if (parent != null)
			{
				if (parent.GetId() == null)
				{
					btree.GetPersister().SaveNode(parent);
				}
				parentOid = (NeoDatis.Odb.OID)parent.GetId();
			}
			else
			{
				parentOid = null;
			}
		}

		public override bool HasParent()
		{
			return parentOid != null;
		}

		protected override void Init()
		{
			childrenOids = new NeoDatis.Odb.OID[maxNbChildren];
			parentOid = null;
			parent = null;
		}

		public override object GetId()
		{
			return oid;
		}

		public override void SetId(object id)
		{
			this.oid = (NeoDatis.Odb.OID)id;
		}

		public override void Clear()
		{
			base.Clear();
			parent = null;
			parentOid = null;
			childrenOids = null;
			oid = null;
		}

		public override void DeleteChildAt(int index)
		{
			childrenOids[index] = null;
			nbChildren--;
		}

		public override void MoveChildFromTo(int sourceIndex, int destinationIndex, bool 
			throwExceptionIfDoesNotExist)
		{
			if (throwExceptionIfDoesNotExist && childrenOids[sourceIndex] == null)
			{
				throw new NeoDatis.Btree.Exception.BTreeException("Trying to load null child node at index "
					 + sourceIndex);
			}
			childrenOids[destinationIndex] = childrenOids[sourceIndex];
		}

		public override void SetChildAt(NeoDatis.Btree.IBTreeNode node, int childIndex, int
			 indexDestination, bool throwExceptionIfDoesNotExist)
		{
			NeoDatis.Odb.OID childOid = (NeoDatis.Odb.OID)node.GetChildIdAt(childIndex, throwExceptionIfDoesNotExist
				);
			childrenOids[indexDestination] = childOid;
			if (childOid != null)
			{
				// The parent of the child has changed
				NeoDatis.Btree.IBTreeNode child = btree.GetPersister().LoadNodeById(childOid);
				child.SetParent(this);
				btree.GetPersister().SaveNode(child);
			}
		}

		public override void SetNullChildAt(int childIndex)
		{
			childrenOids[childIndex] = null;
		}

		public override object GetChildIdAt(int childIndex, bool throwExceptionIfDoesNotExist
			)
		{
			if (throwExceptionIfDoesNotExist && childrenOids[childIndex] == null)
			{
				throw new NeoDatis.Btree.Exception.BTreeException("Trying to load null child node at index "
					 + childIndex);
			}
			return childrenOids[childIndex];
		}

		public override object GetParentId()
		{
			return parentOid;
		}

		public override object GetValueAsObjectAt(int index)
		{
			return GetValueAt(index);
		}
	}
}
