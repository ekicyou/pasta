using NeoDatis.Odb.Test.Update.Nullobject;
using NeoDatis.Odb.Test.VO.Country;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Cyclic
{
	[TestFixture]
    public class TestCyclicReference : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase("cyclic.neodatis");
			NeoDatis.Odb.ODB odb = Open("cyclic.neodatis");
			for (int i = 0; i < 1; i++)
			{
				City brasilia = new City
					("Brasilia" + i);
				Country2 brasil = new Country2
					("Brasil" + i);
				brasilia.SetCountry(brasil);
				brasil.SetCapital(brasilia);
				brasil.SetPopulation(450000);
				odb.Store(brasil);
			}
			odb.Store(new User("name", "email", new Profile("profile")));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("cyclic.neodatis");
			NeoDatis.Odb.Objects<Country2> l = odb.GetObjects<Country2>(true);
			Country2 country = l.GetFirst();
			AssertEquals("Brasil0", country.GetName());
			AssertEquals("Brasilia0", country.GetCapital().GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test15()
		{
			Println("-------------------");
			// LogUtil.logOn(ObjectWriter.LOG_ID, true);
			// LogUtil.logOn(ObjectReader.LOG_ID, true);
			NeoDatis.Odb.ODB odb = Open("cyclic.neodatis");
			NeoDatis.Odb.Objects<Country2> l = odb.GetObjects<Country2(true);
			Country2 country = l.GetFirst();
			City city = country.GetCapital();
			city.SetName("rio de janeiro");
			country.SetCapital(city);
			odb.Store(country);
			odb.Close();
			odb = Open("cyclic.neodatis");
			l = odb.GetObjects<Country2>(true);
			country = l.GetFirst();
			AssertEquals("rio de janeiro", country.GetCapital().GetName());
			l = odb.GetObjects<City>(new CriteriaQuery(Where.Equal("name", "rio de janeiro")));
			AssertEquals(1, l.Count);
			l = odb.GetObjects<City>(new CriteriaQuery());
			AssertEquals(1, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			NeoDatis.Odb.ODB odb = Open("cyclic.neodatis");
			NeoDatis.Odb.Objects<Country2> l = odb.GetObjects<Country2>(true);
			Country2 country = l.GetFirst();
			City city = new City("rio de janeiro");
			country.SetCapital(city);
			odb.Store(country);
			odb.Close();
			odb = Open("cyclic.neodatis");
			l = odb.GetObjects<Country2>(true);
			country = (Country2)l.GetFirst();
			AssertEquals("rio de janeiro", country.GetCapital().GetName());
			l = odb.GetObjects<City>(new CriteriaQuery(Where.Equal("name", "rio de janeiro")));
			AssertEquals(1, l.Count);
			l = odb.GetObjects<City>(new CriteriaQuery());
			AssertEquals(2, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestUniqueInstance1()
		{
			NeoDatis.Odb.ODB odb = Open("cyclic.neodatis");
			NeoDatis.Odb.Objects<City> cities = odb.GetObjects<City>(true);
			NeoDatis.Odb.Objects<Country2> countries = odb.GetObjects<Country2>(true);
			Country2 country = countries.GetFirst();
			City city = cities.GetFirst();
			AssertTrue(country == city.GetCountry());
			AssertTrue(city == country.GetCities()[0]);
			AssertTrue(city == country.GetCapital());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestUniqueInstance2()
		{
			NeoDatis.Odb.ODB odb = Open("cyclic.neodatis");
			NeoDatis.Odb.Objects<Country2> countries = odb.GetObjects<Country2>(true);
			NeoDatis.Odb.Objects<City> cities = odb.GetObjects<City>(true);
			Country2 country = countries.GetFirst();
			City city = cities.GetFirst();
			AssertTrue(country == city.GetCountry());
			AssertTrue(city == country.GetCities()[0]);
			AssertTrue(city == country.GetCapital());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test10()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				string baseName = GetBaseName();
				DeleteBase(baseName);
				odb = Open(baseName);
				ClassA ca = new ClassA();
				ClassB cb = new ClassB(ca, "b");
				ca.SetClassb(cb);
				ca.SetName("a");
				odb.Store(ca);
				ClassInfo ci = Engine.Dummy.GetEngine(odb).GetSession(true).GetMetaModel().GetClassInfo(typeof(NeoDatis.Odb.Test.Cyclic.ClassA
					).FullName, true);
				AssertTrue(ci.HasCyclicReference());
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test11()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open("cyclic.neodatis");
				ClassInfo ci = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
					.GetEngine(odb).GetSession(true).GetMetaModel().GetClassInfo(typeof(NeoDatis.Odb.Test.VO.Login.User
					).FullName, true);
				AssertFalse(ci.HasCyclicReference());
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("cyclic.neodatis");
		}
	}
}
