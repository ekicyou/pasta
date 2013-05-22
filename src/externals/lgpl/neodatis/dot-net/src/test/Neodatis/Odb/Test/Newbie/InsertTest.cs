using NeoDatis.Odb.Test.Newbie.VO;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using System;
using NUnit.Framework;
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
		[Test]
        public virtual void TestInsert()
		{
			DeleteBase(NewbieOdb);
			odb = Open(NewbieOdb);
			Driver marcelo = new Driver
				("marcelo");
			Car car = new Car("car1", 
				4, "ranger", marcelo);
			Car car1 = new Car("car2"
				, 2, "porche");
			Car car2 = new Car("car3"
				, 2, "fusca");
			Car car3 = new Car("car4"
				, 4, "opala");
			Car car4 = new Car("car5"
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
				NeoDatis.Odb.Objects<Car> cars = odb.GetObjects<Car>();
				AssertEquals("The objects weren't added correctly", 5, cars.Count);
				// find for a specific car object
				CriteriaQuery query = new CriteriaQuery(typeof(Car), Where.Equal("name", "car1"));
				cars = odb.GetObjects<Car>(query);
				AssertEquals("The objects couldn't be found correctly", 1, cars.Count);
				// find for a specific composition
				query = new CriteriaQuery(typeof(Car), Where.Equal("driver.name", "marcelo"));
				cars = odb.GetObjects<Car>(query);
				AssertEquals("The objects couldn't be found correctly", 2, cars.Count);
				odb.Commit();
				odb.Close();
				DeleteBase(NewbieOdb);
			}
			catch (System.Exception e)
			{
                Console.WriteLine(e);
			}
		}
	}
}
