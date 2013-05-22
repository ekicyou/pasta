using NUnit.Framework;
namespace NeoDatis.Odb.Test.Btree.Odb
{
	public class MultipleKeyBTree : NeoDatis.Odb.Test.ODBTest
	{
		[Test]
        public virtual void Test1()
		{
			int size = 100000;
			NeoDatis.Btree.IBTree tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("test1", 50);
			for (int i = 0; i < size; i++)
			{
				if (i % 10000 == 0)
				{
					Println(i);
				}
				tree.Insert(i + 1, "value " + (i + 1));
			}
			AssertEquals(size, tree.GetSize());
			System.Collections.IEnumerator iterator = new NeoDatis.Btree.BTreeIteratorMultipleValuesPerKey
				(tree, NeoDatis.Odb.Core.OrderByConstants.OrderByAsc);
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

		[Test]
        public virtual void Test2SameKey()
		{
			int size = 1000;
			int size2 = 100;
			NeoDatis.Btree.IBTree tree = new NeoDatis.Btree.Impl.Multiplevalue.InMemoryBTreeMultipleValuesPerKey
				("test1", 50);
			for (int i = 0; i < size; i++)
			{
				if (i % 10000 == 0)
				{
					Println(i);
				}
				tree.Insert(i + 1, "value " + (i + 1));
			}
			for (int i = 0; i < size2; i++)
			{
				tree.Insert(100, "value " + (i + 1));
			}
			AssertEquals(size + size2, tree.GetSize());
		}
	}
}
