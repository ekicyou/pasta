namespace NeoDatis.Odb.Test.Btree.Odb
{
	/// <author>olivier</author>
	public class SingleValueBTree : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test2SameKeySingleBTree()
		{
			int size = 1000;
			NeoDatis.Btree.IBTree tree = new NeoDatis.Btree.Impl.Singlevalue.InMemoryBTreeSingleValuePerKey
				("test1", 50);
			for (int i = 0; i < size; i++)
			{
				if (i % 10000 == 0)
				{
					Println(i);
				}
				tree.Insert(i + 1, "value " + (i + 1));
			}
			try
			{
				for (int i = 0; i < 10; i++)
				{
					if (i % 10000 == 0)
					{
						Println(i);
					}
					tree.Insert(100, "value " + (i + 1));
					Fail("Single Value Btree should not accept duplcited key");
				}
			}
			catch (System.Exception)
			{
			}
		}
		// TODO: handle exception
	}
}
