namespace NeoDatis.Btree.Tool
{
	public class BTreeValidator
	{
		private static bool on = false;

		public static bool IsOn()
		{
			return NeoDatis.Btree.Tool.BTreeValidator.on;
		}

		public static void SetOn(bool on)
		{
			NeoDatis.Btree.Tool.BTreeValidator.on = on;
		}

		public static void CheckDuplicateChildren(NeoDatis.Btree.IBTreeNode node1, NeoDatis.Btree.IBTreeNode
			 node2)
		{
			if (!on)
			{
				return;
			}
			for (int i = 0; i < node1.GetNbChildren(); i++)
			{
				NeoDatis.Btree.IBTreeNode child1 = node1.GetChildAt(i, true);
				for (int j = 0; j < node2.GetNbChildren(); j++)
				{
					if (child1 == node2.GetChildAt(j, true))
					{
						throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Duplicated node : "
							 + child1);
					}
				}
			}
		}

		public static void ValidateNode(NeoDatis.Btree.IBTreeNode node, bool isRoot)
		{
			if (!on)
			{
				return;
			}
			ValidateNode(node);
			if (isRoot && node.HasParent())
			{
				throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Root node with a parent: "
					 + node.ToString());
			}
			if (!isRoot && !node.HasParent())
			{
				throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Internal node without parent: "
					 + node.ToString());
			}
		}

		public static void ValidateNode(NeoDatis.Btree.IBTreeNode node)
		{
			if (!on)
			{
				return;
			}
			int nbKeys = node.GetNbKeys();
			if (node.HasParent() && nbKeys < node.GetDegree() - 1)
			{
				throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Node with less than "
					 + (node.GetDegree() - 1) + " keys");
			}
			int maxNbKeys = node.GetDegree() * 2 - 1;
			int nbChildren = node.GetNbChildren();
			int maxNbChildren = node.GetDegree() * 2;
			if (nbChildren != 0 && nbKeys == 0)
			{
				throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Node with no key but with children : "
					 + node);
			}
			for (int i = 0; i < nbKeys; i++)
			{
				if (node.GetKeyAndValueAt(i) == null)
				{
					throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Null key at " + 
						i + " on node " + node.ToString());
				}
				CheckValuesOfChild(node.GetKeyAndValueAt(i), node.GetChildAt(i, false));
			}
			for (int i = nbKeys; i < maxNbKeys; i++)
			{
				if (node.GetKeyAndValueAt(i) != null)
				{
					throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Not Null key at "
						 + i + " on node " + node.ToString());
				}
			}
			NeoDatis.Btree.IBTreeNode previousNode = null;
			for (int i = 0; i < nbChildren; i++)
			{
				if (node.GetChildAt(i, false) == null)
				{
					throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Null child at index "
						 + i + " on node " + node.ToString());
				}
				if (previousNode != null && previousNode == node.GetChildAt(i, false))
				{
					throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Two equals children at index "
						 + i + " : " + previousNode.ToString());
				}
				previousNode = node.GetChildAt(i, false);
			}
			for (int i = nbChildren; i < maxNbChildren; i++)
			{
				if (node.GetChildAt(i, false) != null)
				{
					throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Not Null child at "
						 + i + " on node " + node.ToString());
				}
			}
		}

		private static void CheckValuesOfChild(NeoDatis.Btree.IKeyAndValue key, NeoDatis.Btree.IBTreeNode
			 node)
		{
			if (!on)
			{
				return;
			}
			if (node == null)
			{
				return;
			}
			for (int i = 0; i < node.GetNbKeys(); i++)
			{
				if (node.GetKeyAndValueAt(i).GetKey().CompareTo(key.GetKey()) >= 0)
				{
					throw new NeoDatis.Btree.Exception.BTreeNodeValidationException("Left child with values bigger than pivot "
						 + key + " : " + node.ToString());
				}
			}
		}

		public static bool SearchKey(System.IComparable key, NeoDatis.Btree.IBTreeNodeOneValuePerKey
			 node)
		{
			if (!on)
			{
				return false;
			}
			for (int i = 0; i < node.GetNbKeys(); i++)
			{
				if (node.GetKeyAndValueAt(i).GetKey().CompareTo(key) == 0)
				{
					return true;
				}
			}
			return false;
		}
	}
}
