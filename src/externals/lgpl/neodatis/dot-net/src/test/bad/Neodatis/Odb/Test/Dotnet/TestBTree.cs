namespace NeoDatis.Odb.Test.Dotnet
{
	public class TestBTree
	{
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		public virtual void RunJava()
		{
			NeoDatis.Odb.ODB odb = null;
			NeoDatis.Odb.Test.ODBTest test = new NeoDatis.Odb.Test.ODBTest();
			try
			{
				test.DeleteBase("mydb7.neodatis");
				// Open the database
				odb = test.Open("mydb7.neodatis");
				long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				int nRecords = 100000;
				for (int i = 0; i < nRecords; i++)
				{
					NeoDatis.Odb.Test.Dotnet.AA ao = new NeoDatis.Odb.Test.Dotnet.AA();
					ao.ccc = "csdcsdc";
					ao.ww = i;
					odb.Store(ao);
				}
				long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Dotnet.AA));
				query.OrderByAsc("ww");
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Odb.Objects object12 = odb.GetObjects(query, false);
				while (object12.HasNext())
				{
					NeoDatis.Odb.Test.Dotnet.AA s = (NeoDatis.Odb.Test.Dotnet.AA)object12.Next();
					int id = s.ww;
				}
				// println(id);
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				test.Println("Time=" + (end - start) + " / " + (end - start0) + " / " + (end0 - start0
					));
			}
			finally
			{
				if (odb != null)
				{
					// Close the database
					odb.Close();
				}
			}
		}
	}

	internal class aa
	{
		public string ccc;

		public int ww;
	}
}
