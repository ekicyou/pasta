using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using NeoDatis.Odb.Test.Newbie.VO;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Newbie
{
	/// <summary>It is just a simple test to help the newbies</summary>
	/// <author>mayworm at <xmpp://mayworm@gmail.com></author>
	public class UpdateTest : NeoDatis.Odb.Test.ODBTest
	{
		protected static readonly string NewbieOdb = "newbie.neodatis";

		protected static NeoDatis.Odb.ODB odb;

		/// <summary>
		/// This method is considering the
		/// <see cref="NeoDatis.Odb.Test.Newbie.InsertTest">NeoDatis.Odb.Test.Newbie.InsertTest
		/// 	</see>
		/// , so it should be called
		/// in a correct order
		/// </summary>
		[Test]
        public virtual void TestUpdate()
		{
			try
			{
				DeleteBase(NewbieOdb);
				odb = Open(NewbieOdb);
				Driver marcelo = new Driver("marcelo");
				Car car = new Car("car1", 4, "ranger", marcelo);
				odb.Store(car);
				CriteriaQuery query = new CriteriaQuery(Where.Equal("driver.name", "marcelo"));
				Car newCar = (Car)odb.GetObjects<Car>(query).GetFirst();
				newCar.SetDriver(new Driver("dani"));
				odb.Store(newCar);
				odb.Commit();
				query = new CriteriaQuery(Where.Equal("driver.name", "dani"));
				AssertEquals(1, odb.GetObjects<Car>(query).Count);
				odb.Close();
				DeleteBase(NewbieOdb);
			}
			catch (System.Exception e)
			{
                throw e;
			}
		}
	}
}
