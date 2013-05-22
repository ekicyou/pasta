using NeoDatis.Odb.Test.VO.Human;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	[TestFixture]
    public class TestPolyMorphic : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			DeleteBase("multi");
			NeoDatis.Odb.ODB odb = Open("multi");
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine"));
			odb.Close();
			odb = Open("multi");
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(object));
			q.SetPolymorphic(true);
			NeoDatis.Odb.Objects<object> os = odb.GetObjects<object>(q);
			Println(os);
			odb.Close();
			AssertEquals(4, os.Count);
			DeleteBase("multi");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			DeleteBase("multi");
			NeoDatis.Odb.ODB odb = Open("multi");
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine"));
			odb.Close();
			odb = Open("multi");
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Human.Human));
			q.SetPolymorphic(true);
			NeoDatis.Odb.Objects<Human> os = odb.GetObjects<Human>(q);
			Println(os);
			odb.Close();
			AssertEquals(2, os.Count);
			DeleteBase("multi");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3()
		{
			DeleteBase("multi");
			NeoDatis.Odb.ODB odb = Open("multi");
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine"));
			odb.Close();
			odb = Open("multi");
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(object)).Field("specie");
			q.SetPolymorphic(true);
			NeoDatis.Odb.Values os = odb.GetValues(q);
			Println(os);
			odb.Close();
			AssertEquals(4, os.Count);
			DeleteBase("multi");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test4()
		{
			DeleteBase("multi");
			NeoDatis.Odb.ODB odb = Open("multi");
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine"));
			odb.Close();
			odb = Open("multi");
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Human.Human)).Field("specie");
			q.SetPolymorphic(true);
			NeoDatis.Odb.Values os = odb.GetValues(q);
			Println(os);
			odb.Close();
			AssertEquals(2, os.Count);
			DeleteBase("multi");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test5()
		{
			DeleteBase("multi");
			NeoDatis.Odb.ODB odb = Open("multi");
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine"));
			odb.Close();
			odb = Open("multi");
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Human.Man)).Field("specie");
			q.SetPolymorphic(true);
			NeoDatis.Odb.Values os = odb.GetValues(q);
			Println(os);
			odb.Close();
			AssertEquals(1, os.Count);
			DeleteBase("multi");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test6()
		{
			DeleteBase("multi");
			NeoDatis.Odb.ODB odb = Open("multi");
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe"));
			odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine"));
			odb.Close();
			odb = Open("multi");
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(object));
			q.SetPolymorphic(true);
			System.Decimal nb = odb.Count(q);
			Println(nb);
			odb.Close();
			AssertEquals(new System.Decimal(4), nb);
			DeleteBase("multi");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test7()
		{
			int size = isLocal ? 3000 : 300;
            string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog", "M", "my dog"));
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat", "F", "my cat"));
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe" + i));
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine" + i));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(object));
			q.SetPolymorphic(true);
			System.Decimal nb = odb.Count(q);
			Println(nb);
			odb.Close();
			AssertEquals(new System.Decimal(4 * size), nb);
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test8()
		{
			int size = isLocal ? 3000 : 300;
            string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			for (int i = 0; i < size; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("dog" + i, "M", "my dog" + i));
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Animal("cat" + i, "F", "my cat" + i));
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Man("Joe" + i));
				odb.Store(new NeoDatis.Odb.Test.VO.Human.Woman("Karine" + i));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(object), NeoDatis.Odb.Core.Query.Criteria.Where.Equal("specie", "man"));
			q.SetPolymorphic(true);
			System.Decimal nb = odb.Count(q);
			Println(nb);
			odb.Close();
			AssertEquals(new System.Decimal(1 * size), nb);
			DeleteBase(baseName);
		}
	}
}
