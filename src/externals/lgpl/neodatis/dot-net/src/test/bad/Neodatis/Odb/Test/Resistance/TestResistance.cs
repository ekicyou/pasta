namespace NeoDatis.Odb.Test.Resistance
{
	public class TestResistance : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test8()
		{
			int size1 = 1000;
			int size2 = 1000;
			if (!runAll)
			{
				return;
			}
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			NeoDatis.Odb.Objects os = null;
			for (int i = 0; i < size1; i++)
			{
				odb = Open(baseName);
				for (int j = 0; j < size2; j++)
				{
					NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function("function "
						 + j);
					odb.Store(f);
				}
				odb.Close();
				odb = Open(baseName);
				os = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function));
				while (os.HasNext())
				{
					NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)os.Next
						();
					odb.Delete(f);
				}
				odb.Close();
				if (i % 100 == 0)
				{
					Println(i + "/" + size1);
				}
			}
			odb = Open(baseName);
			os = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function));
			AssertEquals(0, os.Count);
			odb.Close();
			Println("step2");
			for (int i = 0; i < size1; i++)
			{
				odb = Open(baseName);
				os = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function));
				while (os.HasNext())
				{
					NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)os.Next
						();
					odb.Delete(f);
				}
				odb.Close();
				odb = Open(baseName);
				for (int j = 0; j < size2; j++)
				{
					NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function("function "
						 + j);
					odb.Store(f);
				}
				odb.Close();
				if (i % 100 == 0)
				{
					Println(i + "/" + size1);
				}
			}
			odb = Open(baseName);
			os = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function));
			AssertEquals(size2, os.Count);
			odb.Close();
			DeleteBase(baseName);
		}
	}
}
