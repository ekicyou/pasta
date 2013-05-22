namespace NeoDatis.Odb.Test.Cyclic
{
	public class TestCyclicReference2 : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase("cyclic.neodatis");
			NeoDatis.Odb.ODB odb = Open("cyclic.neodatis");
			NeoDatis.Odb.Test.VO.Country.Country brasil = new NeoDatis.Odb.Test.VO.Country.Country
				("Brasil");
			for (int i = 0; i < 10; i++)
			{
				NeoDatis.Odb.Test.VO.Country.City city = new NeoDatis.Odb.Test.VO.Country.City("city"
					 + i);
				city.SetCountry(brasil);
				brasil.AddCity(city);
			}
			odb.Store(brasil);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("cyclic.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Country.Country
				), true);
			NeoDatis.Odb.Test.VO.Country.Country country = (NeoDatis.Odb.Test.VO.Country.Country
				)l.GetFirst();
			AssertEquals("Brasil", country.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			Println("-------------------");
			// LogUtil.logOn(ObjectWriter.LOG_ID, true);
			// LogUtil.logOn(ObjectReader.LOG_ID, true);
			NeoDatis.Odb.ODB odb = Open("cyclic.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Country.Country
				), true);
			NeoDatis.Odb.Test.VO.Country.Country country = (NeoDatis.Odb.Test.VO.Country.Country
				)l.GetFirst();
			AssertEquals(10, country.GetCities().Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("cyclic.neodatis");
		}
	}
}
