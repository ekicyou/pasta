namespace NeoDatis.Odb.Test.Query.Values
{
	public class TestGetValuesHandlerParameter : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase("valuesA1");
			NeoDatis.Odb.ODB odb = Open("valuesA1");
			NeoDatis.Odb.Test.Query.Values.Handler handler = new NeoDatis.Odb.Test.Query.Values.Handler
				();
			for (int i = 0; i < 10; i++)
			{
				handler.AddParameter(new NeoDatis.Odb.Test.Query.Values.Parameter("test " + i, "value"
					 + i));
			}
			odb.Store(handler);
			odb.Close();
			odb = Open("valuesA1");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Values.Handler)).Field("parameters"));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			System.Collections.IList l = (System.Collections.IList)ov.GetByAlias("parameters"
				);
			AssertEquals(10, l.Count);
			odb.Close();
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			DeleteBase("valuesA1");
			NeoDatis.Odb.ODB odb = Open("valuesA1");
			NeoDatis.Odb.Test.Query.Values.Handler handler = new NeoDatis.Odb.Test.Query.Values.Handler
				();
			for (int i = 0; i < 10; i++)
			{
				handler.AddParameter(new NeoDatis.Odb.Test.Query.Values.Parameter("test " + i, "value"
					 + i));
			}
			odb.Store(handler);
			odb.Close();
			odb = Open("valuesA1");
			// ValuesQuery in getObjects
			try
			{
				NeoDatis.Odb.Objects objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
					(typeof(NeoDatis.Odb.Test.Query.Values.Handler)).Field("parameters"));
				Fail("Should throw exception");
			}
			catch (System.Exception)
			{
			}
			// TODO: handle exception
			odb.Close();
		}
	}
}
