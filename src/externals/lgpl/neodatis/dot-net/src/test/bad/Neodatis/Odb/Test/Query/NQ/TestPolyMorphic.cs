namespace NeoDatis.Odb.Test.Query.NQ
{
	public class TestPolyMorphic : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			DeleteBase("multi");
			NeoDatis.Odb.ODB odb = Open("multi");
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine"));
			odb.Close();
			odb = Open("multi");
			NeoDatis.Odb.Core.Query.IQuery q = new _SimpleNativeQuery_31();
			q.SetPolymorphic(true);
			NeoDatis.Odb.Objects os = odb.GetObjects(q);
			Println(os);
			odb.Close();
			AssertEquals(4, os.Count);
			DeleteBase("multi");
		}

		private sealed class _SimpleNativeQuery_31 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_31()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Human.Animal animal)
			{
				return true;
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			if (!isLocal)
			{
				return;
			}
			DeleteBase("multi");
			NeoDatis.Odb.ODB odb = Open("multi");
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine"));
			odb.Close();
			odb = Open("multi");
			NeoDatis.Odb.Core.Query.IQuery q = new _SimpleNativeQuery_60();
			q.SetPolymorphic(true);
			NeoDatis.Odb.Objects os = odb.GetObjects(q);
			Println(os);
			odb.Close();
			AssertEquals(2, os.Count);
			DeleteBase("multi");
		}

		private sealed class _SimpleNativeQuery_60 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_60()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Human.Human human)
			{
				return true;
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test8()
		{
			if (!isLocal)
			{
				return;
			}
			int size = isLocal ? 3000 : 300;
			DeleteBase("multi");
			NeoDatis.Odb.ODB odb = Open("multi");
			for (int i = 0; i < size; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog" + i));
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat" + i));
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe" + i));
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("my Karine" + i));
			}
			odb.Close();
			odb = Open("multi");
			NeoDatis.Odb.Core.Query.IQuery q = new _SimpleNativeQuery_91();
			q.SetPolymorphic(true);
			NeoDatis.Odb.Objects objects = odb.GetObjects(q);
			odb.Close();
			DeleteBase("multi");
			AssertEquals(size * 3, objects.Count);
		}

		private sealed class _SimpleNativeQuery_91 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_91()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Human.Animal @object)
			{
				return @object.GetName().StartsWith("my ");
			}
		}
	}
}
