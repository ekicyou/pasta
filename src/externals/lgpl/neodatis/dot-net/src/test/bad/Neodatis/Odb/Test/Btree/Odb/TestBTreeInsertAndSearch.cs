namespace NeoDatis.Odb.Test.Btree.Odb
{
	public class TestBTreeInsertAndSearch : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void TestInsertUsingInt1()
		{
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("default", 5);
			tree.Insert(50, "50");
			tree.Insert(40, "40");
			tree.Insert(30, "30");
			tree.Insert(20, "20");
			tree.Insert(10, "10");
			tree.Insert(15, "15");
			tree.Insert(25, "25");
			tree.Insert(35, "35");
			tree.Insert(21, "21");
			tree.Insert(22, "22");
			tree.Insert(23, "23");
			System.Collections.IList l = tree.Search(22);
			AssertEquals("22", l[0]);
		}

		public virtual void TestInsertUsingInt2()
		{
			int size = 8000;
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("default", 5);
			for (int i = 1; i < size; i++)
			{
				tree.Insert(i, i.ToString());
			}
			System.Collections.IList l = tree.Search(1);
			AssertEquals("[1]", l.ToString());
			l = tree.Search(1000);
			AssertEquals("1000", l[0]);
			l = tree.Search(2000);
			AssertEquals("2000", l[0]);
			l = tree.Search(9800);
			AssertNull(l);
			l = tree.Search(99999);
			AssertEquals(null, l);
		}

		public virtual void TestString1()
		{
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("default", 5);
			tree.Insert("50", "50");
			tree.Insert("40", "40");
			tree.Insert("30", "30");
			tree.Insert("20", "20");
			tree.Insert("10", "10");
			tree.Insert("15", "15");
			tree.Insert("25", "25");
			tree.Insert("35", "35");
			tree.Insert("21", "21");
			tree.Insert("22", "22");
			tree.Insert("23", "23");
			System.Collections.IList p = tree.Search("22");
			AssertEquals("22", p[0]);
		}

		public virtual void TestString2()
		{
			int size = 300;
			int max = 0;
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("default", 5);
			for (int i = 1; i < size; i++)
			{
				for (int j = 1; j < size; j++)
				{
					string key = ((i + 1) * size + j).ToString();
					string value = (i * j).ToString();
					tree.Insert(key, value);
					if (i * j > max)
					{
						max = i * j;
					}
				}
			}
			// println("max = " + max);
			for (int i = 1; i < size; i++)
			{
				for (int j = 1; j < size; j++)
				{
					string key = ((i + 1) * size + j).ToString();
					string value = (i * j).ToString();
					System.Collections.IList p = tree.Search(key);
					AssertEquals(value, p[0]);
				}
			}
		}

		public virtual void Test1()
		{
			int degree = 3;
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("test1", degree);
			tree.Insert(1, "Value 1");
			tree.Insert(20, "Value 20");
			tree.Insert(25, "Value 25");
			tree.Insert(29, "Value 29");
			tree.Insert(21, "Value 21");
			AssertEquals(5, tree.GetRoot().GetNbKeys());
			AssertEquals(0, tree.GetRoot().GetNbChildren());
			AssertEquals(21, tree.GetRoot().GetMedian().GetKey());
			AssertEquals("[Value 21]", tree.GetRoot().GetMedian().GetValue().ToString());
			AssertEquals(0, tree.GetRoot().GetNbChildren());
			// println(tree.getRoot());
			tree.Insert(45, "Value 45");
			AssertEquals(2, tree.GetRoot().GetNbChildren());
			AssertEquals(1, tree.GetRoot().GetNbKeys());
			AssertEquals(21, tree.GetRoot().GetKeyAt(0));
			AssertEquals("[Value 21]", tree.GetRoot().GetValueAsObjectAt(0).ToString());
			// println(tree.getRoot());
			System.Collections.IList o = tree.Search(20);
			AssertEquals("Value 20", o[0]);
			o = tree.Search(29);
			AssertEquals("Value 29", o[0]);
			o = tree.Search(45);
			AssertEquals("Value 45", o[0]);
		}

		public virtual void Test2()
		{
			int degree = 10;
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("test2", degree);
			for (int i = 0; i < 50000; i++)
			{
				tree.Insert(i, "Value " + i);
			}
			AssertEquals("Value 0", tree.Search(0)[0]);
			AssertEquals("Value 1000", tree.Search(1000)[0]);
			AssertEquals("Value 2000", tree.Search(2000)[0]);
			AssertEquals("Value 3000", tree.Search(3000)[0]);
			// tree.resetNbRead();
			AssertEquals("Value 4999", tree.Search(4999)[0]);
		}

		// println("Nb reads = " + tree.getNbRead());
		// println("root = " + tree.getRoot().keysToString(false));
		// println("root[0] = " +
		// tree.getRoot().getChild(0).keysToString(false));
		// println("root[1] = " +
		// tree.getRoot().getChild(1).keysToString(false));
		// println("root[5] = " +
		// tree.getRoot().getChild(3).keysToString(false));
		public virtual void Test3()
		{
			int degree = 3;
			NeoDatis.Btree.IBTree tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("test3", degree);
			tree.Insert(1, "A");
			// tree.insert(new Integer(2),"B");
			tree.Insert(3, "C");
			tree.Insert(4, "D");
			tree.Insert(5, "E");
			// tree.insert(new Integer(6),"F");
			tree.Insert(7, "G");
			// tree.insert(new Integer(8),"H");
			// tree.insert(new Integer(9),"I");
			tree.Insert(10, "J");
			tree.Insert(11, "K");
			// tree.insert(new Integer(12),"L");
			tree.Insert(13, "M");
			tree.Insert(14, "N");
			tree.Insert(15, "O");
			tree.Insert(16, "P");
			// tree.insert(new Integer(17),"Q");
			tree.Insert(18, "R");
			tree.Insert(19, "S");
			tree.Insert(20, "T");
			tree.Insert(21, "U");
			tree.Insert(22, "V");
			// tree.insert(new Integer(23),"W");
			tree.Insert(24, "X");
			tree.Insert(25, "Y");
			tree.Insert(26, "Z");
		}

		// assertEquals(4, tree.getRoot().getNbKeys());
		// assertEquals(0, tree.getRoot().getNbChildren());
		// assertEquals(21, tree.getRoot().getMedianKey());
		// assertEquals("Value 21", tree.getRoot().getMedianValue());
		// assertEquals(0, tree.getRoot().getNbChildren());
		public virtual void Test4()
		{
			int degree = 3;
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree1 = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("1", degree);
			tree1.Insert(1, "A");
			// tree.insert(new Integer(2),"B");
			tree1.Insert(3, "C");
			tree1.Insert(4, "D");
			tree1.Insert(5, "E");
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree2 = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("2", degree);
			tree2.Insert(10, "J");
			tree2.Insert(11, "K");
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree3 = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("3", degree);
			tree3.Insert(14, "N");
			tree3.Insert(15, "O");
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree4 = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("4", degree);
			tree4.Insert(18, "R");
			tree4.Insert(19, "S");
			tree4.Insert(20, "T");
			tree4.Insert(21, "U");
			tree4.Insert(22, "V");
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree5 = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("5", degree);
			tree5.Insert(25, "Y");
			tree5.Insert(26, "Z");
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree6 = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("6", degree);
			tree6.Insert(7, "G");
			tree6.Insert(13, "M");
			tree6.Insert(16, "P");
			tree6.Insert(24, "X");
			tree6.GetRoot().SetChildAt(tree1.GetRoot(), 0);
			tree6.GetRoot().SetChildAt(tree2.GetRoot(), 1);
			tree6.GetRoot().SetChildAt(tree3.GetRoot(), 2);
			tree6.GetRoot().SetChildAt(tree4.GetRoot(), 3);
			tree6.GetRoot().SetChildAt(tree5.GetRoot(), 4);
			tree6.GetRoot().SetNbChildren(5);
			// println("Test 4");
			tree6.Insert(2, "B");
			// println(tree6.getRoot().getChild(0).keysToString(true));
			AssertEquals("[B]", tree6.GetRoot().GetChildAt(0, true).GetValueAsObjectAt(1).ToString
				());
			tree6.Insert(17, "Q");
			// println(tree6.getRoot().keysToString(true));
			AssertEquals(5, tree6.GetRoot().GetNbKeys());
			// println(tree6.getRoot().getChild(3).keysToString(true));
			AssertEquals("[Q]", tree6.GetRoot().GetChildAt(3, true).GetValueAsObjectAt(0).ToString
				());
			AssertEquals("[R]", tree6.GetRoot().GetChildAt(3, true).GetValueAsObjectAt(1).ToString
				());
			AssertEquals("[S]", tree6.GetRoot().GetChildAt(3, true).GetValueAsObjectAt(2).ToString
				());
			// println(tree6.getRoot().getChild(4).keysToString(true));
			AssertEquals("[U]", tree6.GetRoot().GetChildAt(4, true).GetValueAsObjectAt(0).ToString
				());
			AssertEquals("[V]", tree6.GetRoot().GetChildAt(4, true).GetValueAsObjectAt(1).ToString
				());
			tree6.Insert(12, "L");
			// println(tree6.getRoot().keysToString(true));
			AssertEquals(1, tree6.GetRoot().GetNbKeys());
			AssertEquals(2, tree6.GetRoot().GetChildAt(0, true).GetNbKeys());
			// println(tree6.getRoot().getChild(0).keysToString(true));
			AssertEquals("[G]", tree6.GetRoot().GetChildAt(0, true).GetValueAsObjectAt(0).ToString
				());
			AssertEquals("[M]", tree6.GetRoot().GetChildAt(0, true).GetValueAsObjectAt(1).ToString
				());
			// println(tree6.getRoot().getChild(0).getChild(1).keysToString(true));
			AssertEquals("[J]", tree6.GetRoot().GetChildAt(0, true).GetChildAt(1, true).GetValueAsObjectAt
				(0).ToString());
			AssertEquals("[K]", tree6.GetRoot().GetChildAt(0, true).GetChildAt(1, true).GetValueAsObjectAt
				(1).ToString());
			AssertEquals("[L]", tree6.GetRoot().GetChildAt(0, true).GetChildAt(1, true).GetValueAsObjectAt
				(2).ToString());
			tree6.Insert(6, "F");
			// println(tree6.getRoot().keysToString(true));
			AssertEquals(1, tree6.GetRoot().GetNbKeys());
			AssertEquals(3, tree6.GetRoot().GetChildAt(0, true).GetNbKeys());
			AssertEquals(2, tree6.GetRoot().GetChildAt(0, true).GetChildAt(0, true).GetNbKeys
				());
			// println(tree6.getRoot().getChild(0).getChild(0).keysToString(true));
			AssertEquals("[A]", tree6.GetRoot().GetChildAt(0, true).GetChildAt(0, true).GetValueAsObjectAt
				(0).ToString());
			AssertEquals("[B]", tree6.GetRoot().GetChildAt(0, true).GetChildAt(0, true).GetValueAsObjectAt
				(1).ToString());
			// println(tree6.getRoot().getChild(1).getChild(2).keysToString(true));
			AssertEquals("[Z]", tree6.GetRoot().GetChildAt(1, true).GetChildAt(2, true).GetValueAsObjectAt
				(1).ToString());
		}

		public virtual void Test5()
		{
			int degree = 40;
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("5", degree);
			long a0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < 500000; i++)
			{
				tree.Insert(i, "Value " + i);
			}
			long a1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("insert time = " + (a1 - a0));
			AssertEquals("[Value 0]", tree.Search(0).ToString());
			AssertEquals("[Value 1000]", tree.Search(1000).ToString());
			AssertEquals("[Value 2000]", tree.Search(2000).ToString());
			AssertEquals("[Value 48000]", tree.Search(48000).ToString());
			// tree.resetNbRead();
			long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < 100000; i++)
			{
				AssertEquals("[Value 490000]", tree.Search(490000).ToString());
			}
			long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// tree.resetNbRead();
			AssertEquals("[Value 490000]", tree.Search(490000).ToString());
		}

		// println("Test5 compl- Nb reads = " + tree.getNbRead()+ " -
		// nb comp="+IntKeyBTree.getNbComparison()+ " - t="+(t1-t0));
		public virtual void TestNonUniqueKey()
		{
			int degree = 3;
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree1 = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("7", degree);
			tree1.Insert(1, "A");
			tree1.Insert(1, "AA");
			tree1.Insert(1, "AAA");
			AssertEquals(3, tree1.Search(1).Count);
			AssertEquals("[A, AA, AAA]", tree1.Search(1).ToString());
			AssertEquals(3, tree1.GetSize());
		}

		public virtual void TestNonUniqueKey2()
		{
			int degree = 3;
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree1 = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("7", degree);
			tree1.Insert(1, "A");
			tree1.Insert(1, "AA");
			tree1.Insert(1, "AAA");
			tree1.Insert(1, "BBB");
			System.Collections.ICollection c = tree1.Search(1);
			AssertEquals(4, c.Count);
			System.Collections.IEnumerator iterator = c.GetEnumerator();
			AssertEquals("A", iterator.Current);
			AssertEquals("AA", iterator.Current);
			AssertEquals(4, tree1.GetSize());
			AssertEquals("[A, AA, AAA, BBB]", c.ToString());
		}
	}
}
