namespace NeoDatis.Odb.Test.Instantiationhelper
{
	/// <summary>
	/// Test if the ODB retrieves objects without default constructor and null
	/// arguments
	/// </summary>
	/// <author>mayworm at <xmpp://mayworm@gmail.com></author>
	public class TestInstanceHelper : NeoDatis.Odb.Test.ODBTest
	{
		private NeoDatis.Odb.ODB odb;

		private static readonly string Testdbname = "instanceHelper.neodatis";

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase(Testdbname);
			odb = Open(Testdbname);
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase(Testdbname);
		}

		/// <summary>Create, store and try retrieve the object without default constructor</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestUseInstanceHelper()
		{
			NeoDatis.Odb.OdbConfiguration.SetEnableEmptyConstructorCreation(false);
			try
			{
				NeoDatis.Odb.ClassRepresentation carRepresentation = odb.GetClassRepresentation(typeof(
					NeoDatis.Odb.Test.Instantiationhelper.Car));
				// create a db and store a object that has not default constructor
				NeoDatis.Odb.Test.Instantiationhelper.Car car = new NeoDatis.Odb.Test.Instantiationhelper.Car
					("Ranger", 2006);
				odb.Store(car);
				odb.Commit();
				CloseAndReopenDb();
				NeoDatis.Odb.Objects cars;
				try
				{
					CheckCarRetrieval();
					Fail("Expected exception");
				}
				catch (NeoDatis.Odb.ODBRuntimeException)
				{
				}
				// expected
				CloseAndReopenDb();
				carRepresentation.AddInstantiationHelper(new _InstantiationHelper_76());
				CheckCarRetrieval();
				CloseAndReopenDb();
				carRepresentation.RemoveInstantiationHelper();
				carRepresentation.AddParameterHelper(new _ParameterHelper_84());
				try
				{
					CheckCarRetrieval();
					Fail("Expected Exception");
				}
				catch (NeoDatis.Odb.ODBRuntimeException)
				{
				}
				// expected
				odb.Close();
			}
			finally
			{
				NeoDatis.Odb.OdbConfiguration.SetEnableEmptyConstructorCreation(true);
			}
		}

		private sealed class _InstantiationHelper_76 : NeoDatis.Odb.Core.Layers.Layer2.Instance.InstantiationHelper
		{
			public _InstantiationHelper_76()
			{
			}

			public object Instantiate()
			{
				return new NeoDatis.Odb.Test.Instantiationhelper.Car("dummyModel", 1);
			}
		}

		private sealed class _ParameterHelper_84 : NeoDatis.Odb.Core.Layers.Layer2.Instance.ParameterHelper
		{
			public _ParameterHelper_84()
			{
			}

			public object[] Parameters()
			{
				return new object[0];
			}
		}

		/// <summary>Create, store and try retrieve the object without default constructor</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestWithoutHelperUsingNoConstructor()
		{
			// create a db and store a object that has not default constructor
			NeoDatis.Odb.Test.Instantiationhelper.Car car = new NeoDatis.Odb.Test.Instantiationhelper.Car
				("Ranger", 2006);
			odb.Store(car);
			CloseAndReopenDb();
			NeoDatis.Odb.Objects cars;
			CheckCarRetrieval();
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		private void CheckCarRetrieval()
		{
			NeoDatis.Odb.Objects cars = odb.GetObjects(typeof(NeoDatis.Odb.Test.Instantiationhelper.Car
				));
			AssertEquals(1, cars.Count);
			NeoDatis.Odb.Test.Instantiationhelper.Car car = (NeoDatis.Odb.Test.Instantiationhelper.Car
				)cars.GetFirst();
			AssertEquals(car.GetModel(), "Ranger");
			AssertEquals(car.GetYear(), 2006);
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Instantiationhelper.Car), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("model", "Ranger"));
			cars = odb.GetObjects(query);
			car = (NeoDatis.Odb.Test.Instantiationhelper.Car)cars.GetFirst();
			AssertEquals(car.GetModel(), "Ranger");
		}

		/// <exception cref="System.Exception"></exception>
		private void CloseAndReopenDb()
		{
			odb.Close();
			odb = Open(Testdbname);
		}
	}
}
