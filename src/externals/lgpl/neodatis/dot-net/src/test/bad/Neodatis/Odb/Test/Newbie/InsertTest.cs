namespace NeoDatis.Odb.Test.Newbie
{
	/// <summary>It is just a simple test to help the newbies</summary>
	/// <author>mayworm at <xmpp://mayworm@gmail.com></author>
	public class InsertTest : NeoDatis.Odb.Test.ODBTest
	{
		protected static readonly string NewbieOdb = "newbie.neodatis";

		protected static NeoDatis.Odb.ODB odb;

		/// <summary>Insert different objects on database</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		/// <exception cref="System.IO.IOException">System.IO.IOException</exception>
		public virtual void TestInsert()
		{
			DeleteBase(NewbieOdb);
			odb = Open(NewbieOdb);
			NeoDatis.Odb.Test.Newbie.VO.Driver marcelo = new NeoDatis.Odb.Test.Newbie.VO.Driver
				("marcelo");
			NeoDatis.Odb.Test.Newbie.VO.Car car = new NeoDatis.Odb.Test.Newbie.VO.Car("car1", 
				4, "ranger", marcelo);
			NeoDatis.Odb.Test.Newbie.VO.Car car1 = new NeoDatis.Odb.Test.Newbie.VO.Car("car2"
				, 2, "porche");
			NeoDatis.Odb.Test.Newbie.VO.Car car2 = new NeoDatis.Odb.Test.Newbie.VO.Car("car3"
				, 2, "fusca");
			NeoDatis.Odb.Test.Newbie.VO.Car car3 = new NeoDatis.Odb.Test.Newbie.VO.Car("car4"
				, 4, "opala");
			NeoDatis.Odb.Test.Newbie.VO.Car car4 = new NeoDatis.Odb.Test.Newbie.VO.Car("car5"
				, 4, "vectra", marcelo);
			try
			{
				// open is called on NewbieTest
				// insert 5 car's
				odb.Store(car);
				odb.Store(car1);
				odb.Store(car2);
				odb.Store(car3);
				odb.Store(car4);
				// find for all car objects
				NeoDatis.Odb.Objects cars = odb.GetObjects(typeof(NeoDatis.Odb.Test.Newbie.VO.Car
					));
				AssertEquals("The objects weren't added correctly", 5, cars.Count);
				// find for a specific car object
				NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Newbie.VO.Car), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "car1"));
				cars = odb.GetObjects(query);
				AssertEquals("The objects couldn't be found correctly", 1, cars.Count);
				// find for a specific composition
				query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(NeoDatis.Odb.Test.Newbie.VO.Car
					), NeoDatis.Odb.Core.Query.Criteria.Where.Equal("driver.name", "marcelo"));
				cars = odb.GetObjects(query);
				AssertEquals("The objects couldn't be found correctly", 2, cars.Count);
				odb.Commit();
				odb.Close();
				DeleteBase(NewbieOdb);
			}
			catch (System.Exception e)
			{
				Sharpen.Runtime.PrintStackTrace(e);
			}
		}
	}
}
