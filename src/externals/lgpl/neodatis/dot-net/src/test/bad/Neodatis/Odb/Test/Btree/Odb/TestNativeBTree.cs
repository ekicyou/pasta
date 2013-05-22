namespace NeoDatis.Odb.Test.Btree.Odb
{
	public class TestNativeBTree : NeoDatis.Odb.Test.NeoDatisAssert
	{
		public virtual void Test1()
		{
			int size = 100;
			NeoDatis.Btree.IBTree tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("bt1", 2);
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
				j++;
				AssertEquals("value " + j, o);
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
				("bt1", 2);
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
	}
}
