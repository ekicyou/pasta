namespace NeoDatis.Odb.Test.Performance
{
	public class TestComparisonDotNet : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = null;
			try
			{
				DeleteBase("mydb.neodatis");
				// Open the database
				odb = Open("mydb.neodatis");
				long t0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				int nRecords = 100000;
				for (int i = 0; i < nRecords; i++)
				{
					NeoDatis.Odb.Test.Performance.Class1 ao = new NeoDatis.Odb.Test.Performance.Class1
						(189, "csdcsdc");
					odb.Store(ao);
				}
				odb.Close();
				long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				odb = Open("mydb.neodatis");
				NeoDatis.Odb.Objects ssss = odb.GetObjects(typeof(NeoDatis.Odb.Test.Performance.Class1
					));
				long t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				Println("Elapsed time for inserting " + nRecords + " records: " + (t1 - t0) + " / select = "
					 + (t2 - t1));
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
}
