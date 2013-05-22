namespace NeoDatis.Btree
{
	/// <summary>The interface for btree node.</summary>
	/// <remarks>The interface for btree node.</remarks>
	/// <author>olivier</author>
	public interface IBTreeNode
	{
		bool IsFull();

		bool IsLeaf();

		NeoDatis.Btree.IKeyAndValue GetKeyAndValueAt(int index);

		System.IComparable GetKeyAt(int index);

		object GetValueAsObjectAt(int index);

		NeoDatis.Btree.IKeyAndValue GetLastKeyAndValue();

		NeoDatis.Btree.IBTreeNode GetChildAt(int index, bool throwExceptionIfNotExist);

		NeoDatis.Btree.IBTreeNode GetLastChild();

		NeoDatis.Btree.IBTreeNode GetLastPositionChild();

		NeoDatis.Btree.IBTreeNode GetParent();

		object GetParentId();

		void SetKeyAndValueAt(System.IComparable key, object value, int index);

		void SetKeyAndValueAt(NeoDatis.Btree.IKeyAndValue keyAndValue, int index);

		void SetKeyAndValueAt(System.IComparable key, object value, int index, bool shiftIfAlreadyExist
			, bool incrementNbKeys);

		void SetKeyAndValueAt(NeoDatis.Btree.IKeyAndValue keyAndValue, int index, bool shiftIfAlreadyExist
			, bool incrementNbKeys);

		NeoDatis.Btree.IBTreeNode ExtractRightPart();

		NeoDatis.Btree.IKeyAndValue GetMedian();

		void SetChildAt(NeoDatis.Btree.IBTreeNode node, int childIndex, int indexDestination
			, bool throwExceptionIfDoesNotExist);

		void SetChildAt(NeoDatis.Btree.IBTreeNode child, int index);

		void SetNullChildAt(int childIndex);

		void MoveChildFromTo(int sourceIndex, int destinationIndex, bool throwExceptionIfDoesNotExist
			);

		void IncrementNbKeys();

		void IncrementNbChildren();

		/// <summary>Returns the position of the key.</summary>
		/// <remarks>
		/// Returns the position of the key. If the key does not exist in node,
		/// returns the position where this key should be,multiplied by -1
		/// <pre>
		/// or example for node of degree 3 : [1 89 452 789 - ],
		/// calling getPositionOfKey(89) returns 2 (starts with 1)
		/// calling getPositionOfKey(99) returns -2 (starts with 1),because the position should be done, but it does not exist so multiply by -1
		/// his is used to know the child we should descend to!in this case the getChild(2).
		/// </pre>
		/// </remarks>
		/// <param name="key"></param>
		/// <returns>
		/// The position of the key,as a negative number if key does not
		/// exist, warning, the position starts with 1and not 0!
		/// </returns>
		int GetPositionOfKey(System.IComparable key);

		void InsertKeyAndValue(System.IComparable key, object value);

		void MergeWith(NeoDatis.Btree.IBTreeNode node);

		void RemoveKeyAndValueAt(int index);

		int GetNbKeys();

		void SetNbKeys(int nbKeys);

		void SetNbChildren(int nbChildren);

		int GetDegree();

		int GetNbChildren();

		int GetMaxNbChildren();

		void SetParent(NeoDatis.Btree.IBTreeNode node);

		object DeleteKeyForLeafNode(NeoDatis.Btree.IKeyAndValue keyAndValue);

		object DeleteKeyAndValueAt(int index, bool shiftChildren);

		bool HasParent();

		object GetId();

		void SetId(object id);

		void SetBTree(NeoDatis.Btree.IBTree btree);

		NeoDatis.Btree.IBTree GetBTree();

		void Clear();

		void DeleteChildAt(int index);

		object GetChildIdAt(int childIndex, bool throwExceptionIfDoesNotExist);
	}
}
