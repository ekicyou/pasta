namespace NeoDatis.Odb.Test.Btree.Odb
{
	public class TestLazyBTree : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			int size = 100000;
			NeoDatis.Btree.IBTree tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("test1", 2);
			for (int i = 0; i < size; i++)
			{
				tree.Insert(i + 1, "value " + (i + 1));
			}
			AssertEquals(size, tree.GetSize());
			System.Collections.IEnumerator iterator = tree.Iterator(NeoDatis.Odb.Core.OrderByConstants
				.OrderByAsc);
			int j = 0;
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				// println(o);
				j++;
				if (j == size)
				{
					AssertEquals("value " + size, o);
				}
			}
		}

		public virtual void Test2()
		{
			int size = 100000;
			NeoDatis.Btree.IBTree tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("test2", 2);
			for (int i = 0; i < size; i++)
			{
				tree.Insert(i + 1, "value " + (i + 1));
			}
			AssertEquals(size, tree.GetSize());
			System.Collections.IEnumerator iterator = tree.Iterator(NeoDatis.Odb.Core.OrderByConstants
				.OrderByDesc);
			int j = 0;
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				// println(o);
				j++;
				if (j == size)
				{
					AssertEquals("value " + 1, o);
				}
			}
		}

		public virtual void Test3()
		{
			int size = 100000;
			NeoDatis.Btree.IBTree tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("test1", 2);
			for (int i = 0; i < size; i++)
			{
				// println(i);
				tree.Insert((i + 1).ToString(), "value " + (i + 1));
			}
			AssertEquals(size, tree.GetSize());
			System.Collections.IEnumerator iterator = tree.Iterator(NeoDatis.Odb.Core.OrderByConstants
				.OrderByAsc);
			int j = 0;
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				// println(o);
				j++;
				if (j == size)
				{
					AssertEquals("value " + (size - 1), o);
				}
			}
		}
	}
}
