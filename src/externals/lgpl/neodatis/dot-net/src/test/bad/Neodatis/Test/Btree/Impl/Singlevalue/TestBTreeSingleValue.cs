namespace NeoDatis.Test.Btree.Impl.Singlevalue
{
	public class TestBTreeSingleValue : NeoDatis.Odb.Test.NeoDatisAssert
	{
		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(3);
			btree.Insert(1, "key 1");
			AssertEquals(1, btree.GetSize());
			AssertEquals("key 1", btree.Search(1));
			object o = btree.Delete(1, "key 1");
			AssertEquals("key 1", o);
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete2()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(3);
			btree.Insert(1, "key 1");
			btree.Insert(2, "key 2");
			btree.Insert(3, "key 3");
			btree.Insert(4, "key 4");
			btree.Insert(5, "key 5");
			AssertEquals(5, btree.GetSize());
			AssertEquals("key 1", btree.Search(1));
			AssertEquals("key 2", btree.Search(2));
			AssertEquals("key 3", btree.Search(3));
			AssertEquals("key 4", btree.Search(4));
			AssertEquals("key 5", btree.Search(5));
			object o = btree.Delete(1, "key 1");
			AssertEquals("key 1", o);
			o = btree.Delete(2, "key 2");
			AssertEquals("key 2", o);
			o = btree.Delete(3, "key 3");
			AssertEquals("key 3", o);
			o = btree.Delete(4, "key 4");
			AssertEquals("key 4", o);
			o = btree.Delete(5, "key 5");
			AssertEquals("key 5", o);
			AssertEquals(0, btree.GetSize());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
		}

		public virtual void TestDelete25()
		{
			NeoDatis.Btree.IBTree tree = GetBTree(3);
			tree.Insert("A", "A");
			tree.Insert("B", "B");
			tree.Insert("C", "C");
			tree.Insert("D", "D");
			tree.Insert("E", "E");
			tree.Insert("F", "F");
			tree.Insert("G", "G");
			tree.Insert("J", "J");
			tree.Insert("K", "K");
			tree.Insert("L", "L");
			tree.Insert("M", "M");
			tree.Insert("N", "N");
			tree.Insert("O", "O");
			tree.Insert("P", "P");
			tree.Insert("Q", "Q");
			tree.Insert("R", "R");
			tree.Insert("S", "S");
			tree.Insert("T", "T");
			tree.Insert("U", "U");
			tree.Insert("V", "V");
			tree.Insert("X", "X");
			tree.Insert("Y", "Y");
			tree.Insert("Z", "Z");
		}

		public virtual void TestDelete26()
		{
			NeoDatis.Btree.IBTree tree = GetBTree(3);
			tree.Insert("Z", "Z");
			tree.Insert("Y", "Y");
			tree.Insert("X", "X");
			tree.Insert("V", "V");
			tree.Insert("U", "U");
			tree.Insert("T", "T");
			tree.Insert("S", "S");
			tree.Insert("R", "R");
			tree.Insert("Q", "Q");
			tree.Insert("P", "P");
			tree.Insert("O", "O");
			tree.Insert("N", "N");
			tree.Insert("M", "M");
			tree.Insert("L", "L");
			tree.Insert("K", "K");
			tree.Insert("J", "J");
			tree.Insert("G", "G");
			tree.Insert("F", "F");
			tree.Insert("E", "E");
			tree.Insert("D", "D");
			tree.Insert("C", "C");
			tree.Insert("B", "B");
			tree.Insert("A", "A");
			AssertEquals(23, tree.GetSize());
			NeoDatis.Btree.IBTreeNodeOneValuePerKey child1 = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)tree.GetRoot().GetChildAt(0, false);
			NeoDatis.Btree.IBTreeNodeOneValuePerKey child14 = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)child1.GetChildAt(3, false);
			AssertNotNull(child14);
		}

		/// <summary>Cromen example, second edition, page 450</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestDelete27()
		{
			NeoDatis.Btree.IBTree tree = GetBTree(3);
			tree.Insert("P", "P");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey PNode = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)tree.GetRoot();
			NeoDatis.Btree.IBTreeNodeOneValuePerKey CGMNode = GetBTreeNode(tree, "cgm");
			CGMNode.InsertKeyAndValue("C", "C");
			CGMNode.InsertKeyAndValue("G", "G");
			CGMNode.InsertKeyAndValue("M", "M");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey TXNode = GetBTreeNode(tree, "tx");
			TXNode.InsertKeyAndValue("T", "T");
			TXNode.InsertKeyAndValue("X", "X");
			PNode.SetChildAt(CGMNode, 0);
			PNode.SetChildAt(TXNode, 1);
			PNode.SetNbChildren(2);
			NeoDatis.Btree.IBTreeNodeOneValuePerKey ABNode = GetBTreeNode(tree, "ab");
			ABNode.InsertKeyAndValue("A", "A");
			ABNode.InsertKeyAndValue("B", "B");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey DEFode = GetBTreeNode(tree, "def");
			DEFode.InsertKeyAndValue("D", "D");
			DEFode.InsertKeyAndValue("E", "E");
			DEFode.InsertKeyAndValue("F", "F");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey JKLNode = GetBTreeNode(tree, "jkl");
			JKLNode.InsertKeyAndValue("J", "J");
			JKLNode.InsertKeyAndValue("K", "K");
			JKLNode.InsertKeyAndValue("L", "L");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey NONode = GetBTreeNode(tree, "no");
			NONode.InsertKeyAndValue("N", "N");
			NONode.InsertKeyAndValue("O", "O");
			CGMNode.SetChildAt(ABNode, 0);
			CGMNode.SetChildAt(DEFode, 1);
			CGMNode.SetChildAt(JKLNode, 2);
			CGMNode.SetChildAt(NONode, 3);
			CGMNode.SetNbChildren(4);
			NeoDatis.Btree.IBTreeNodeOneValuePerKey QRSNode = GetBTreeNode(tree, "qrs");
			QRSNode.InsertKeyAndValue("Q", "Q");
			QRSNode.InsertKeyAndValue("R", "R");
			QRSNode.InsertKeyAndValue("S", "S");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey UVNode = GetBTreeNode(tree, "uv");
			UVNode.InsertKeyAndValue("U", "U");
			UVNode.InsertKeyAndValue("V", "V");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey YZNode = GetBTreeNode(tree, "yz");
			YZNode.InsertKeyAndValue("Y", "Y");
			YZNode.InsertKeyAndValue("Z", "Z");
			TXNode.SetChildAt(QRSNode, 0);
			TXNode.SetChildAt(UVNode, 1);
			TXNode.SetChildAt(YZNode, 2);
			TXNode.SetNbChildren(3);
			string s1 = "h=1:[P]" + "h=2:[C,G,M][T,X]" + "h=3:[A,B][D,E,F][J,K,L][N,O][Q,R,S][U,V][Y,Z]";
			// case 1
			string s2AfterDeleteingF = "h=1:[P]" + "h=2:[C,G,M][T,X]" + "h=3:[A,B][D,E][J,K,L][N,O][Q,R,S][U,V][Y,Z]";
			object F = tree.Delete("F", "F");
			AssertEquals("F", F);
			string s = new NeoDatis.Btree.Tool.BTreeDisplay().Build(tree.GetRoot(), 3, false)
				.ToString();
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, " ", string.Empty);
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "\n", string.Empty);
			AssertEquals(s2AfterDeleteingF, s);
			// case 2a
			string s2AfterDeleteingM = "h=1:[P]" + "h=2:[C,G,L][T,X]" + "h=3:[A,B][D,E][J,K][N,O][Q,R,S][U,V][Y,Z]";
			object M = tree.Delete("M", "M");
			AssertEquals("M", M);
			s = new NeoDatis.Btree.Tool.BTreeDisplay().Build(tree.GetRoot(), 3, false).ToString
				();
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, " ", string.Empty);
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "\n", string.Empty);
			AssertEquals(s2AfterDeleteingM, s);
			// case 2c
			string s2AfterDeleteingG = "h=1:[P]" + "h=2:[C,L][T,X]" + "h=3:[A,B][D,E,J,K][N,O][Q,R,S][U,V][Y,Z]";
			object G = tree.Delete("G", "G");
			AssertEquals("G", G);
			s = new NeoDatis.Btree.Tool.BTreeDisplay().Build(tree.GetRoot(), 3, false).ToString
				();
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, " ", string.Empty);
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "\n", string.Empty);
			AssertEquals(s2AfterDeleteingG, s);
			// case 3b
			string s2AfterDeleteingD = "h=1:[C,L,P,T,X]" + "h=2:[A,B][E,J,K][N,O][Q,R,S][U,V][Y,Z]";
			object D = tree.Delete("D", "D");
			// assertEquals(2, tree.getHeight());
			AssertEquals("D", D);
			s = new NeoDatis.Btree.Tool.BTreeDisplay().Build(tree.GetRoot(), 3, false).ToString
				();
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, " ", string.Empty);
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "\n", string.Empty);
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "h=3:", string.Empty);
			AssertEquals(s2AfterDeleteingD, s);
			// case 3a
			string s2AfterDeleteingB = "h=1:[E,L,P,T,X]" + "h=2:[A,C][J,K][N,O][Q,R,S][U,V][Y,Z]";
			object B = tree.Delete("B", "B");
			AssertEquals("B", B);
			s = new NeoDatis.Btree.Tool.BTreeDisplay().Build(tree.GetRoot(), 3, false).ToString
				();
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, " ", string.Empty);
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "\n", string.Empty);
			s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, "h=3:", string.Empty);
			AssertEquals(s2AfterDeleteingB, s);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete3()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(3);
			int size = 10;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "key " + i);
			}
			AssertEquals(size, btree.GetSize());
			for (int i = 0; i < size; i++)
			{
				AssertEquals("key " + i, btree.Delete(i, "key " + i));
			}
			AssertEquals(0, btree.GetSize());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete10000()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(20);
			int size = 100000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "key " + i);
			}
			AssertEquals(size, btree.GetSize());
			for (int i = 0; i < size; i++)
			{
				AssertEquals("key " + i, btree.Delete(i, "key " + i));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete100000()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(3);
			int size = 10000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "key " + i);
			}
			AssertEquals(size, btree.GetSize());
			for (int i = 0; i < size; i++)
			{
				AssertEquals("key " + i, btree.Delete(i, "key " + i));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete10000Alpha_2()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(2);
			int size = 10000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert("key" + i, "value " + i);
			}
			object o = btree.Search("key71");
			AssertEquals(size, btree.GetSize());
			for (int i = size - 1; i >= 0; i--)
			{
				// println(new BTreeDisplay().build(btree));
				AssertEquals("value " + i, btree.Delete("key" + i, "value " + i));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete10000Alpha_3()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(3);
			int size = 10000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert("key" + i, "value " + i);
			}
			object o = btree.Search("key71");
			AssertEquals(size, btree.GetSize());
			for (int i = size - 1; i >= 0; i--)
			{
				// println(new BTreeDisplay().build(btree));
				AssertEquals("value " + i, btree.Delete("key" + i, "value " + i));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete100000Alpha_2()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(2);
			int size = 100000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert("key" + i, "value " + i);
			}
			object o = btree.Search("key71");
			AssertEquals(size, btree.GetSize());
			for (int i = size - 1; i >= 0; i--)
			{
				// println(new BTreeDisplay().build(btree));
				AssertEquals("value " + i, btree.Delete("key" + i, "value " + i));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDeleteStringKey()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(3);
			btree.Insert("key70", "70");
			btree.Insert("key71", "71");
			// println(new BTreeDisplay().build(btree));
			AssertEquals("70", btree.GetRoot().GetKeyAndValueAt(0).GetValue());
			AssertEquals("71", btree.GetRoot().GetKeyAndValueAt(1).GetValue());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDeleteStringKey2()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(3);
			btree.Insert("key700", "700");
			btree.Insert("key710", "710");
			btree.Insert("key720", "720");
			btree.Insert("key730", "730");
			btree.Insert("key740", "740");
			btree.Insert("key715", "715");
		}

		// println(new BTreeDisplay().build(btree));
		// assertEquals("70", btree.getRoot().getKeyAndValueAt(0).getValue());
		// assertEquals("71", btree.getRoot().getKeyAndValueAt(1).getValue());
		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete10_3()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(3);
			int size = 10;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "value " + i);
			}
			AssertEquals(size, btree.GetSize());
			for (int i = size - 1; i >= 0; i--)
			{
				AssertEquals("value " + i, btree.Delete(i, "value " + i));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete100_3()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(3);
			int size = 100;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "value " + i);
			}
			AssertEquals(size, btree.GetSize());
			for (int i = size - 1; i >= 0; i--)
			{
				AssertEquals("value " + i, btree.Delete(i, "value " + i));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete1000_3()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(3);
			int size = 1000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "value " + i);
			}
			AssertEquals(size, btree.GetSize());
			for (int i = size - 1; i >= 0; i--)
			{
				AssertEquals("value " + i, btree.Delete(i, "value " + i));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDeleteInsert100000()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(3);
			int size = 200000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "key " + i);
			}
			AssertEquals(size, btree.GetSize());
			for (int i = 0; i < size; i++)
			{
				AssertEquals("key " + i, btree.Delete(i, "key " + i));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
			for (int i = 0; i < size; i++)
			{
				btree.Insert(size + i, "key " + (i + size));
			}
			for (int i = 0; i < size; i++)
			{
				AssertEquals("key " + (i + size), btree.Delete(i + size, "key " + (i + size)));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		public virtual void TestInsert()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(3);
			btree.Insert(1, "key 1");
			AssertEquals(1, btree.GetSize());
			AssertEquals("key 1", btree.Search(1));
		}

		public virtual void TestInsert2()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(3);
			btree.Insert(1, "key 1");
			btree.Insert(2, "key 2");
			btree.Insert(3, "key 3");
			btree.Insert(4, "key 4");
			btree.Insert(5, "key 5");
			AssertEquals(5, btree.GetSize());
			AssertEquals("key 1", btree.Search(1));
			AssertEquals("key 2", btree.Search(2));
			AssertEquals("key 3", btree.Search(3));
			AssertEquals("key 4", btree.Search(4));
			AssertEquals("key 5", btree.Search(5));
		}

		public virtual void TestInsert3()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(3);
			btree.Insert(1, "key 1");
			btree.Insert(2, "key 2");
			btree.Insert(3, "key 3");
			btree.Insert(4, "key 4");
			btree.Insert(5, "key 5");
			btree.Insert(6, "key 6");
			AssertEquals(6, btree.GetSize());
			AssertEquals("key 1", btree.Search(1));
			AssertEquals("key 2", btree.Search(2));
			AssertEquals("key 3", btree.Search(3));
			AssertEquals("key 4", btree.Search(4));
			AssertEquals("key 5", btree.Search(5));
			AssertEquals("key 6", btree.Search(6));
			AssertEquals(2, btree.GetRoot().GetNbChildren());
			// child 1 should be [1,2]
			NeoDatis.Btree.IBTreeNodeOneValuePerKey child1 = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)btree.GetRoot().GetChildAt(0, false);
			AssertEquals(2, child1.GetNbKeys());
			AssertEquals(0, child1.GetNbChildren());
			AssertEquals("key 1", child1.GetKeyAndValueAt(0).GetValue());
			AssertEquals(1, child1.GetKeyAndValueAt(0).GetKey());
			// child 2 should be [4,5,6]
			NeoDatis.Btree.IBTreeNodeOneValuePerKey child2 = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)btree.GetRoot().GetChildAt(1, false);
			AssertEquals(3, child2.GetNbKeys());
			AssertEquals(0, child2.GetNbChildren());
			AssertEquals("key 4", child2.GetKeyAndValueAt(0).GetValue());
			AssertEquals("key 5", child2.GetKeyAndValueAt(1).GetValue());
			AssertEquals("key 6", child2.GetKeyAndValueAt(2).GetValue());
			// child 2 should be null
			NeoDatis.Btree.IBTreeNodeOneValuePerKey child3 = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)btree.GetRoot().GetChildAt(2, false);
			AssertEquals(null, child3);
		}

		private NeoDatis.Btree.IBTreeSingleValuePerKey GetBTree(int degree)
		{
			return new NeoDatis.Btree.Impl.Singlevalue.InMemoryBTreeSingleValuePerKey("default"
				, degree);
		}

		public virtual void Testsearch10()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(3);
			for (int i = 0; i < 10; i++)
			{
				btree.Insert(i, "key " + i);
			}
			AssertEquals(10, btree.GetSize());
			AssertEquals("key 1", btree.Search(1));
			AssertEquals("key 9", btree.Search(9));
			NeoDatis.Btree.IBTreeNodeOneValuePerKey child3 = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)btree.GetRoot().GetChildAt(2, false);
			AssertEquals(4, child3.GetNbKeys());
			AssertEquals(6, child3.GetKeyAt(0));
			AssertEquals(7, child3.GetKeyAt(1));
			AssertEquals(8, child3.GetKeyAt(2));
			AssertEquals(9, child3.GetKeyAt(3));
			AssertEquals(null, child3.GetKeyAt(4));
		}

		public virtual void Testsearch500()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(3);
			for (int i = 0; i < 500; i++)
			{
				btree.Insert(i, "key " + i);
			}
			AssertEquals(500, btree.GetSize());
			AssertEquals("key 1", btree.Search(1));
			AssertEquals("key 499", btree.Search(499));
		}

		public virtual void Testsearch10000()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(10);
			int size = 110000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "key " + i);
			}
			AssertEquals(size, btree.GetSize());
			for (int i = 0; i < size; i++)
			{
				AssertEquals("key " + i, btree.Search(i));
			}
		}

		public virtual void Testsearch500000()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(10);
			int size = 500000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "key " + i);
			}
			AssertEquals(size, btree.GetSize());
			for (int i = 0; i < size; i++)
			{
				AssertEquals("key " + i, btree.Search(i));
			}
		}

		/// <summary>
		/// <pre>
		/// node1 = [    10,     100]
		/// |        |
		/// |        |
		/// c1       c2
		/// c1 = [1,2,3]
		/// result of split should be
		/// node1 = [    2  ,    10,     100]
		/// |        |      |
		/// |        |      |
		/// c1       c1'    c2
		/// where c1 = [1]
		/// and c1'=[3]
		/// </pre>
		/// </summary>
		public virtual void TestSplit()
		{
			NeoDatis.Btree.IBTree tree = GetBTree(2);
			tree.Insert(10, "Key 10");
			tree.Insert(100, "Key 100");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey c1 = GetBTreeNode(tree, "child 1");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey c2 = GetBTreeNode(tree, "child 2");
			NeoDatis.Btree.IBTreeNodeOneValuePerKey node1 = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)tree.GetRoot();
			node1.SetChildAt(c1, 0);
			node1.SetChildAt(c2, 1);
			node1.SetNbKeys(2);
			node1.SetNbChildren(2);
			c1.SetKeyAndValueAt(1, "Key 1", 0);
			c1.SetKeyAndValueAt(2, "Key 2", 1);
			c1.SetKeyAndValueAt(3, "Key 3", 2);
			c1.SetNbKeys(3);
			AssertEquals(0, c1.GetNbChildren());
			tree.Split(node1, c1, 0);
			AssertEquals(3, node1.GetNbKeys());
			AssertEquals(3, node1.GetNbChildren());
			AssertEquals(2, node1.GetKeyAndValueAt(0).GetKey());
			AssertEquals(10, node1.GetKeyAndValueAt(1).GetKey());
			AssertEquals(100, node1.GetKeyAndValueAt(2).GetKey());
			NeoDatis.Btree.IBTreeNodeOneValuePerKey c1New = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)node1.GetChildAt(0, false);
			AssertEquals(1, c1New.GetNbKeys());
			AssertEquals(0, c1New.GetNbChildren());
			AssertEquals(1, c1New.GetKeyAt(0));
			AssertEquals(null, c1New.GetKeyAt(1));
			AssertEquals(null, c1New.GetKeyAt(2));
			AssertEquals(node1, c1New.GetParent());
			NeoDatis.Btree.IBTreeNodeOneValuePerKey c1bis = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)node1.GetChildAt(1, false);
			AssertEquals(1, c1bis.GetNbKeys());
			AssertEquals(0, c1bis.GetNbChildren());
			AssertEquals(3, c1bis.GetKeyAt(0));
			AssertEquals(node1, c1bis.GetParent());
			AssertEquals(null, c1bis.GetKeyAt(1));
			AssertEquals(null, c1bis.GetKeyAt(2));
			NeoDatis.Btree.IBTreeNodeOneValuePerKey c2New = (NeoDatis.Btree.IBTreeNodeOneValuePerKey
				)node1.GetChildAt(2, false);
			AssertEquals(c2, c2New);
		}

		private NeoDatis.Btree.IBTreeNodeOneValuePerKey GetBTreeNode(NeoDatis.Btree.IBTree
			 tree, string name)
		{
			return new NeoDatis.Test.Btree.Impl.Singlevalue.MockBTreeNodeSingleValue(tree, name
				);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestgetBiggestSmallest1()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(3);
			btree.Insert(1, "key 1");
			btree.Insert(2, "key 2");
			btree.Insert(3, "key 3");
			btree.Insert(4, "key 4");
			btree.Insert(5, "key 5");
			AssertEquals(5, btree.GetSize());
			AssertEquals("key 5", btree.GetBiggest(btree.GetRoot(), false).GetValue());
			AssertEquals("key 1", btree.GetSmallest(btree.GetRoot(), false).GetValue());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestgetBiggestSmallest1WithDelete()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(3);
			btree.Insert(1, "key 1");
			btree.Insert(2, "key 2");
			btree.Insert(3, "key 3");
			btree.Insert(4, "key 4");
			btree.Insert(5, "key 5");
			AssertEquals(5, btree.GetSize());
			AssertEquals("key 5", btree.GetBiggest(btree.GetRoot(), true).GetValue());
			AssertEquals("key 1", btree.GetSmallest(btree.GetRoot(), true).GetValue());
			AssertEquals(null, btree.Search(1));
			AssertEquals(null, btree.Search(5));
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestgetBiggestSmallest1WithDelete2()
		{
			NeoDatis.Btree.IBTreeSingleValuePerKey btree = GetBTree(10);
			int size = 500000;
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "key " + i);
			}
			AssertEquals(size, btree.GetSize());
			AssertEquals("key 499999", btree.GetBiggest(btree.GetRoot(), true).GetValue());
			AssertEquals("key 0", btree.GetSmallest(btree.GetRoot(), true).GetValue());
			AssertEquals(null, btree.Search(0));
			AssertEquals(null, btree.Search(499999));
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDelete1()
		{
			NeoDatis.Btree.IBTree btree = GetBTree(10);
			int size = 500000;
			long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				btree.Insert(i, "key " + i);
			}
			long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("insert time=" + (t1-t0));
			AssertEquals(size, btree.GetSize());
			AssertEquals("key 499999", btree.Delete(499999, "key 499999"));
		}
	}
}
