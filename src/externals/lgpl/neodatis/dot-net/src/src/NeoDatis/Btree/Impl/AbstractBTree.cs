namespace NeoDatis.Btree.Impl
{
	[System.Serializable]
	public abstract class AbstractBTree : NeoDatis.Btree.IBTree
	{
		private string name;

		private int degree;

		private long size;

		private int height;

		private NeoDatis.Btree.IBTreeNode root;

		[System.NonSerialized]
		protected NeoDatis.Btree.IBTreePersister persister;

		protected int controlNumber;

		public abstract NeoDatis.Btree.IBTreeNode BuildNode();

		public AbstractBTree()
		{
			this.degree = 0;
			this.size = 0;
			this.height = 1;
			this.persister = null;
			root = null;
		}

		public AbstractBTree(string name, int degree, NeoDatis.Btree.IBTreePersister persister
			)
		{
			this.name = name;
			this.degree = degree;
			this.size = 0;
			this.height = 1;
			this.persister = persister;
			root = BuildNode();
			// TODO check if it is needed to store the root before the btree ->
			// saving btree will try to update root!
			persister.SaveNode(root);
			persister.SaveBTree(this);
			persister.Flush();
		}

		/// <summary>TODO Manage collision</summary>
		public virtual object Delete(System.IComparable key, object value)
		{
			object o = null;
			try
			{
				o = InternalDelete(root, new NeoDatis.Btree.Impl.KeyAndValue(key, value));
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Error while deleting key='"
					 + key + "' value='" + value + "'", e);
			}
			if (o != null)
			{
				size--;
			}
			GetPersister().SaveBTree(this);
			// TODO flush or not?
			// persister.flush();
			return o;
		}

		/// <summary>Returns the value of the deleted key</summary>
		/// <param name="node"></param>
		/// <param name="keyAndValue"></param>
		/// <returns></returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		protected virtual object InternalDelete(NeoDatis.Btree.IBTreeNode node, NeoDatis.Btree.IKeyAndValue
			 keyAndValue)
		{
			int position = node.GetPositionOfKey(keyAndValue.GetKey());
			bool keyIsHere = position > 0;
			int realPosition = -1;
			NeoDatis.Btree.IBTreeNode leftNode = null;
			NeoDatis.Btree.IBTreeNode rightNode = null;
			try
			{
				if (node.IsLeaf())
				{
					if (keyIsHere)
					{
						object deletedValue = node.DeleteKeyForLeafNode(keyAndValue);
						GetPersister().SaveNode(node);
						return deletedValue;
					}
					// key does not exist
					return null;
				}
				if (!keyIsHere)
				{
					// descend
					realPosition = -position - 1;
					NeoDatis.Btree.IBTreeNode child = node.GetChildAt(realPosition, true);
					if (child.GetNbKeys() == degree - 1)
					{
						node = PrepareForDelete(node, child, realPosition);
						return InternalDelete(node, keyAndValue);
					}
					return InternalDelete(child, keyAndValue);
				}
				// Here,the node is not a leaf and contains the key
				realPosition = position - 1;
				System.IComparable currentKey = node.GetKeyAt(realPosition);
				object currentValue = node.GetValueAsObjectAt(realPosition);
				// case 2a
				leftNode = node.GetChildAt(realPosition, true);
				if (leftNode.GetNbKeys() >= degree)
				{
					NeoDatis.Btree.IKeyAndValue prev = GetBiggest(leftNode, true);
					node.SetKeyAndValueAt(prev, realPosition);
					NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(node, node == root);
					GetPersister().SaveNode(node);
					return currentValue;
				}
				// case 2b
				rightNode = node.GetChildAt(realPosition + 1, true);
				if (rightNode.GetNbKeys() >= degree)
				{
					NeoDatis.Btree.IKeyAndValue next = GetSmallest(rightNode, true);
					node.SetKeyAndValueAt(next, realPosition);
					NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(node, node == root);
					GetPersister().SaveNode(node);
					return currentValue;
				}
				// case 2c
				// Here, both left and right part have degree-1 keys
				// remove the element to be deleted from node (shifting left all
				// right
				// elements, link to right link does not exist anymore)
				// insert the key to be deleted in left child and merge the 2 nodes.
				// rightNode should be deleted
				// if node is root, then leftNode becomes the new root and node
				// should be deleted
				// 
				node.DeleteKeyAndValueAt(realPosition, true);
				leftNode.InsertKeyAndValue(currentKey, currentValue);
				leftNode.MergeWith(rightNode);
				// If node is the root and is empty
				if (!node.HasParent() && node.GetNbKeys() == 0)
				{
					persister.DeleteNode(node);
					root = leftNode;
					leftNode.SetParent(null);
					// The height has been decreased. No need to save btree here.
					// The calling delete method will save it.
					height--;
				}
				else
				{
					node.SetChildAt(leftNode, realPosition);
					// Node must only be validated if it is not the root
					NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(node, node == root);
				}
				persister.DeleteNode(rightNode);
				NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(leftNode, leftNode == root);
				GetPersister().SaveNode(node);
				GetPersister().SaveNode(leftNode);
				return InternalDelete(leftNode, keyAndValue);
			}
			finally
			{
			}
		}

		private NeoDatis.Btree.IBTreeNode PrepareForDelete(NeoDatis.Btree.IBTreeNode parent
			, NeoDatis.Btree.IBTreeNode child, int childIndex)
		{
			NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(parent);
			NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(child);
			// case 3a
			NeoDatis.Btree.IBTreeNode leftSibling = null;
			NeoDatis.Btree.IBTreeNode rightSibling = null;
			try
			{
				if (childIndex > 0 && parent.GetNbChildren() > 0)
				{
					leftSibling = parent.GetChildAt(childIndex - 1, false);
				}
				if (childIndex < parent.GetNbChildren() - 1)
				{
					rightSibling = parent.GetChildAt(childIndex + 1, false);
				}
				// case 3a left
				if (leftSibling != null && leftSibling.GetNbKeys() >= degree)
				{
					NeoDatis.Btree.IKeyAndValue elementToMoveDown = parent.GetKeyAndValueAt(childIndex
						 - 1);
					NeoDatis.Btree.IKeyAndValue elementToMoveUp = leftSibling.GetLastKeyAndValue();
					parent.SetKeyAndValueAt(elementToMoveUp, childIndex - 1);
					child.InsertKeyAndValue(elementToMoveDown.GetKey(), elementToMoveDown.GetValue());
					if (leftSibling.GetNbChildren() > leftSibling.GetNbKeys())
					{
						// Take the last child of the left sibling and set it the
						// first child of the 'child' (incoming parameter)
						// child.setChildAt(leftSibling.getChildAt(leftSibling.getNbChildren()
						// - 1, true), 0);
						child.SetChildAt(leftSibling, leftSibling.GetNbChildren() - 1, 0, true);
						child.IncrementNbChildren();
					}
					leftSibling.DeleteKeyAndValueAt(leftSibling.GetNbKeys() - 1, false);
					if (!leftSibling.IsLeaf())
					{
						leftSibling.DeleteChildAt(leftSibling.GetNbChildren() - 1);
					}
					persister.SaveNode(parent);
					persister.SaveNode(child);
					persister.SaveNode(leftSibling);
					if (NeoDatis.Btree.Tool.BTreeValidator.IsOn())
					{
						NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(parent, parent == root);
						NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(child, false);
						NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(leftSibling, false);
						NeoDatis.Btree.Tool.BTreeValidator.CheckDuplicateChildren(leftSibling, child);
					}
					return parent;
				}
				// case 3a right
				if (rightSibling != null && rightSibling.GetNbKeys() >= degree)
				{
					NeoDatis.Btree.IKeyAndValue elementToMoveDown = parent.GetKeyAndValueAt(childIndex
						);
					NeoDatis.Btree.IKeyAndValue elementToMoveUp = rightSibling.GetKeyAndValueAt(0);
					parent.SetKeyAndValueAt(elementToMoveUp, childIndex);
					child.InsertKeyAndValue(elementToMoveDown.GetKey(), elementToMoveDown.GetValue());
					if (rightSibling.GetNbChildren() > 0)
					{
						// Take the first child of the right sibling and set it the
						// last child of the 'child' (incoming parameter)
						child.SetChildAt(rightSibling, 0, child.GetNbChildren(), true);
						child.IncrementNbChildren();
					}
					rightSibling.DeleteKeyAndValueAt(0, true);
					persister.SaveNode(parent);
					persister.SaveNode(child);
					persister.SaveNode(rightSibling);
					if (NeoDatis.Btree.Tool.BTreeValidator.IsOn())
					{
						NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(parent, parent == root);
						NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(child, false);
						NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(rightSibling, false);
						NeoDatis.Btree.Tool.BTreeValidator.CheckDuplicateChildren(rightSibling, child);
					}
					return parent;
				}
				// case 3b
				bool isCase3b = (leftSibling != null && leftSibling.GetNbKeys() == degree - 1) ||
					 (rightSibling != null && rightSibling.GetNbKeys() >= degree - 1);
				bool parentWasSetToNull = false;
				if (isCase3b)
				{
					// choose left sibling to execute merge
					if (leftSibling != null)
					{
						NeoDatis.Btree.IKeyAndValue elementToMoveDown = parent.GetKeyAndValueAt(childIndex
							 - 1);
						leftSibling.InsertKeyAndValue(elementToMoveDown.GetKey(), elementToMoveDown.GetValue
							());
						leftSibling.MergeWith(child);
						parent.DeleteKeyAndValueAt(childIndex - 1, true);
						if (parent.GetNbKeys() == 0)
						{
							// this is the root
							if (!parent.HasParent())
							{
								root = leftSibling;
								root.SetParent(null);
								height--;
								parentWasSetToNull = true;
							}
							else
							{
								throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Unexpected empty node that is node the root!"
									);
							}
						}
						else
						{
							parent.SetChildAt(leftSibling, childIndex - 1);
						}
						if (parentWasSetToNull)
						{
							persister.DeleteNode(parent);
						}
						else
						{
							persister.SaveNode(parent);
							NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(parent, parent == root);
						}
						// child was merged with another node it must be deleted
						persister.DeleteNode(child);
						persister.SaveNode(leftSibling);
						// Validator.validateNode(child, child == root);
						NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(leftSibling, leftSibling == root);
						// Validator.checkDuplicateChildren(leftSibling, child);
						if (parentWasSetToNull)
						{
							return root;
						}
						return parent;
					}
					// choose right sibling to execute merge
					if (rightSibling != null)
					{
						NeoDatis.Btree.IKeyAndValue elementToMoveDown = parent.GetKeyAndValueAt(childIndex
							);
						child.InsertKeyAndValue(elementToMoveDown.GetKey(), elementToMoveDown.GetValue());
						child.MergeWith(rightSibling);
						parent.DeleteKeyAndValueAt(childIndex, true);
						if (parent.GetNbKeys() == 0)
						{
							// this is the root
							if (!parent.HasParent())
							{
								root = child;
								root.SetParent(null);
								height--;
								parentWasSetToNull = true;
							}
							else
							{
								throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Unexpected empty root node!"
									);
							}
						}
						else
						{
							parent.SetChildAt(child, childIndex);
						}
						if (parentWasSetToNull)
						{
							persister.DeleteNode(parent);
						}
						else
						{
							persister.SaveNode(parent);
							NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(parent, parent == root);
						}
						persister.DeleteNode(rightSibling);
						persister.SaveNode(child);
						NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(child, child == root);
						// Validator.validateNode(rightSibling, rightSibling ==
						// root);
						// Validator.checkDuplicateChildren(rightSibling, child);
						if (parentWasSetToNull)
						{
							return root;
						}
						return parent;
					}
					throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("deleting case 3b but no non null sibling!"
						);
				}
			}
			finally
			{
			}
			throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Unexpected case in executing prepare for delete"
				);
		}

		public virtual int GetDegree()
		{
			return degree;
		}

		public virtual void Insert(System.IComparable key, object value)
		{
			// check if root is full
			if (root.IsFull())
			{
				NeoDatis.Btree.IBTreeNode newRoot = BuildNode();
				NeoDatis.Btree.IBTreeNode oldRoot = root;
				newRoot.SetChildAt(root, 0);
				newRoot.SetNbChildren(1);
				root = newRoot;
				Split(newRoot, oldRoot, 0);
				height++;
				persister.SaveNode(oldRoot);
				// TODO Remove the save of the new root : the save on the btree
				// should do the save on the new root(after introspector
				// refactoring)
				persister.SaveNode(newRoot);
				persister.SaveBTree(this);
				NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(newRoot, true);
			}
			InsertNonFull(root, key, value);
			size++;
			persister.SaveBTree(this);
		}

		// Commented by Olivier 05/11/2007
		// persister.flush();
		private void InsertNonFull(NeoDatis.Btree.IBTreeNode node, System.IComparable key
			, object value)
		{
			if (node.IsLeaf())
			{
				node.InsertKeyAndValue(key, value);
				persister.SaveNode(node);
				return;
			}
			int position = node.GetPositionOfKey(key);
			// return an index starting
			// from 1 instead of 0
			int realPosition = -position - 1;
			// If position is positive, the key must be inserted in this node
			if (position >= 0)
			{
				realPosition = position - 1;
				node.InsertKeyAndValue(key, value);
				persister.SaveNode(node);
				return;
			}
			// descend
			NeoDatis.Btree.IBTreeNode nodeToDescend = node.GetChildAt(realPosition, true);
			if (nodeToDescend.IsFull())
			{
				Split(node, nodeToDescend, realPosition);
				if (node.GetKeyAt(realPosition).CompareTo(key) < 0)
				{
					nodeToDescend = node.GetChildAt(realPosition + 1, true);
				}
			}
			InsertNonFull(nodeToDescend, key, value);
		}

		/// <summary>
		/// <pre>
		/// 1 take median element
		/// 2 insert the median in the parent  (shifting necessary elements)
		/// 3 create a new node with right part elements (moving keys and values and children)
		/// 4 set this new node as a child of parent
		/// </pre>
		/// </summary>
		public virtual void Split(NeoDatis.Btree.IBTreeNode parent, NeoDatis.Btree.IBTreeNode
			 node2Split, int childIndex)
		{
			// BTreeValidator.validateNode(parent, parent == root);
			// BTreeValidator.validateNode(node2Split, false);
			// 1
			NeoDatis.Btree.IKeyAndValue median = node2Split.GetMedian();
			// 2
			parent.SetKeyAndValueAt(median, childIndex, true, true);
			// 3
			NeoDatis.Btree.IBTreeNode rightPart = node2Split.ExtractRightPart();
			// 4
			parent.SetChildAt(rightPart, childIndex + 1);
			parent.SetChildAt(node2Split, childIndex);
			parent.IncrementNbChildren();
			persister.SaveNode(parent);
			persister.SaveNode(rightPart);
			persister.SaveNode(node2Split);
			if (NeoDatis.Btree.Tool.BTreeValidator.IsOn())
			{
				NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(parent, parent == root);
				NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(rightPart, false);
				NeoDatis.Btree.Tool.BTreeValidator.ValidateNode(node2Split, false);
			}
		}

		public virtual long GetSize()
		{
			return size;
		}

		public virtual NeoDatis.Btree.IBTreeNode GetRoot()
		{
			return root;
		}

		public virtual int GetHeight()
		{
			return height;
		}

		public virtual NeoDatis.Btree.IBTreePersister GetPersister()
		{
			return persister;
		}

		public virtual void SetPersister(NeoDatis.Btree.IBTreePersister persister)
		{
			this.persister = persister;
			this.persister.SetBTree(this);
			if (root.GetBTree() == null)
			{
				root.SetBTree(this);
			}
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void Clear()
		{
			root.Clear();
		}

		public virtual NeoDatis.Btree.IKeyAndValue GetBiggest(NeoDatis.Btree.IBTreeNode node
			, bool delete)
		{
			int lastKeyIndex = node.GetNbKeys() - 1;
			int lastChildIndex = node.GetNbChildren() - 1;
			if (lastChildIndex > lastKeyIndex)
			{
				NeoDatis.Btree.IBTreeNode child = node.GetChildAt(lastChildIndex, true);
				if (child.GetNbKeys() == degree - 1)
				{
					node = PrepareForDelete(node, child, lastChildIndex);
				}
				lastChildIndex = node.GetNbChildren() - 1;
				child = node.GetChildAt(lastChildIndex, true);
				return GetBiggest(child, delete);
			}
			NeoDatis.Btree.IKeyAndValue kav = node.GetKeyAndValueAt(lastKeyIndex);
			if (delete)
			{
				node.DeleteKeyAndValueAt(lastKeyIndex, false);
				persister.SaveNode(node);
			}
			return kav;
		}

		public virtual NeoDatis.Btree.IKeyAndValue GetSmallest(NeoDatis.Btree.IBTreeNode 
			node, bool delete)
		{
			if (!node.IsLeaf())
			{
				NeoDatis.Btree.IBTreeNode child = node.GetChildAt(0, true);
				if (child.GetNbKeys() == degree - 1)
				{
					node = PrepareForDelete(node, child, 0);
				}
				child = node.GetChildAt(0, true);
				return GetSmallest(child, delete);
			}
			NeoDatis.Btree.IKeyAndValue kav = node.GetKeyAndValueAt(0);
			if (delete)
			{
				node.DeleteKeyAndValueAt(0, true);
				persister.SaveNode(node);
			}
			return kav;
		}

		public abstract object GetId();

		public abstract System.Collections.IEnumerator Iterator<T>(NeoDatis.Odb.Core.OrderByConstants
			 arg1);

		public abstract void SetId(object arg1);
	}
}
