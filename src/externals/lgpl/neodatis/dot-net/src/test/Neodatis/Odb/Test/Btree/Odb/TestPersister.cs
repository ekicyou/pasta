using NUnit.Framework;
namespace NeoDatis.Odb.Test.Btree.Odb
{
	[TestFixture]
    public class TestPersister : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			DeleteBase("btree45.neodatis");
			NeoDatis.Odb.ODB odb = Open("btree45.neodatis");
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(odb);
			NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister persister = new NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister
				(storageEngine);
			NeoDatis.Btree.IBTreeMultipleValuesPerKey tree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeMultiple
				("t", 3, persister);
			tree.Insert(1, new NeoDatis.Odb.Test.Btree.Odb.MyObject("Value 1"));
			tree.Insert(20, new NeoDatis.Odb.Test.Btree.Odb.MyObject("Value 20"));
			tree.Insert(25, new NeoDatis.Odb.Test.Btree.Odb.MyObject("Value 25"));
			tree.Insert(29, new NeoDatis.Odb.Test.Btree.Odb.MyObject("Value 29"));
			tree.Insert(21, new NeoDatis.Odb.Test.Btree.Odb.MyObject("Value 21"));
			AssertEquals(5, tree.GetRoot().GetNbKeys());
			AssertEquals(0, tree.GetRoot().GetNbChildren());
			AssertEquals(21, tree.GetRoot().GetMedian().GetKey());
			AssertEquals("[Value 21]", tree.GetRoot().GetMedian().GetValue().ToString());
			AssertEquals(0, tree.GetRoot().GetNbChildren());
			// println(tree.getRoot());
			tree.Insert(45, new NeoDatis.Odb.Test.Btree.Odb.MyObject("Value 45"));
			AssertEquals(2, tree.GetRoot().GetNbChildren());
			AssertEquals(1, tree.GetRoot().GetNbKeys());
			AssertEquals(21, tree.GetRoot().GetKeyAt(0));
			AssertEquals("[Value 21]", tree.GetRoot().GetValueAsObjectAt(0).ToString());
			persister.Close();
			odb = Open("btree45.neodatis");
			storageEngine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine(odb);
			persister = new NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister(storageEngine);
			tree = (NeoDatis.Btree.IBTreeMultipleValuesPerKey)persister.LoadBTree(tree.GetId(
				));
			AssertEquals(6, tree.GetSize());
			// println(tree.getRoot());
			NeoDatis.Odb.Test.Btree.Odb.MyObject o = (NeoDatis.Odb.Test.Btree.Odb.MyObject)tree
				.Search(20)[0];
			AssertEquals("Value 20", o.GetName());
			o = (NeoDatis.Odb.Test.Btree.Odb.MyObject)tree.Search(29)[0];
			AssertEquals("Value 29", o.GetName());
			o = (NeoDatis.Odb.Test.Btree.Odb.MyObject)tree.Search(45)[0];
			AssertEquals("Value 45", o.GetName());
			odb.Close();
			DeleteBase("btree45.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestDirectSave()
		{
			DeleteBase("btree46.neodatis");
			NeoDatis.Odb.ODB odb = Open("btree46.neodatis");
			NeoDatis.Btree.IBTree tree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeMultiple("t"
				, 3, new NeoDatis.Btree.Impl.InMemoryPersister());
			NeoDatis.Btree.IBTreeNodeMultipleValuesPerKey node = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeNodeMultiple
				(tree);
			odb.Store(node);
			for (int i = 0; i < 4; i++)
			{
				node.SetKeyAndValueAt(new NeoDatis.Btree.Impl.KeyAndValue(i + 1, "String" + (i + 
					1)), i);
				odb.Store(node);
			}
			odb.Close();
			DeleteBase("btree46.neodatis");
		}
	}

	internal class MyObject
	{
		private string name;

		public MyObject(string name) : base()
		{
			this.name = name;
		}

		public virtual string GetName()
		{
			return name;
		}

		public override string ToString()
		{
			return name;
		}
	}
}
