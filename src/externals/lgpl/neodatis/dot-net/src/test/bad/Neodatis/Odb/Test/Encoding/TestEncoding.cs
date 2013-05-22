namespace NeoDatis.Odb.Test.Encoding
{
	public class TestEncoding : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="Java.IO.UnsupportedEncodingException"></exception>
		public virtual void Test1()
		{
			string baseName = GetBaseName();
			Println(baseName);
			string currentEncoding = NeoDatis.Odb.OdbConfiguration.GetDatabaseCharacterEncoding
				();
			NeoDatis.Odb.OdbConfiguration.SetDatabaseCharacterEncoding("ISO8859-5");
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(baseName);
				string nameWithCyrillicCharacters = "\u0410 \u0430 \u0431 \u0448 \u0429";
				NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function(nameWithCyrillicCharacters
					);
				NeoDatis.Odb.OID oid = odb.Store(f);
				odb.Close();
				Println(f);
				odb = Open(baseName);
				NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)odb
					.GetObjectFromId(oid);
				odb.Close();
				AssertEquals(nameWithCyrillicCharacters, f2.GetName());
				AssertEquals('\u0410', f2.GetName()[0]);
				AssertEquals('\u0430', f2.GetName()[2]);
				AssertEquals('\u0431', f2.GetName()[4]);
				AssertEquals('\u0448', f2.GetName()[6]);
				AssertEquals('\u0429', f2.GetName()[8]);
			}
			finally
			{
				NeoDatis.Odb.OdbConfiguration.SetDatabaseCharacterEncoding(currentEncoding);
			}
		}

		/// <exception cref="Java.IO.UnsupportedEncodingException"></exception>
		/// <exception cref="System.Exception"></exception>
		public virtual void Test2_ClientServer()
		{
			string baseName = GetBaseName();
			Println(baseName);
			string currentEncoding = NeoDatis.Odb.OdbConfiguration.GetDatabaseCharacterEncoding
				();
			NeoDatis.Odb.OdbConfiguration.SetDatabaseCharacterEncoding("ISO8859-5");
			NeoDatis.Odb.ODBServer server = null;
			try
			{
				server = NeoDatis.Odb.ODBFactory.OpenServer(NeoDatis.Odb.Test.ODBTest.Port + 1);
				server.AddBase(baseName, baseName);
				server.StartServer(true);
				Java.Lang.Thread.Sleep(200);
				NeoDatis.Odb.ODB odb = NeoDatis.Odb.ODBFactory.OpenClient("localhost", NeoDatis.Odb.Test.ODBTest
					.Port + 1, baseName);
				string nameWithCyrillicCharacters = "\u0410 \u0430 \u0431 \u0448 \u0429";
				NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function(nameWithCyrillicCharacters
					);
				NeoDatis.Odb.OID oid = odb.Store(f);
				odb.Close();
				Println(f);
				odb = NeoDatis.Odb.ODBFactory.OpenClient("localhost", NeoDatis.Odb.Test.ODBTest.Port
					 + 1, baseName);
				NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)odb
					.GetObjectFromId(oid);
				odb.Close();
				AssertEquals(nameWithCyrillicCharacters, f2.GetName());
				AssertEquals('\u0410', f2.GetName()[0]);
				AssertEquals('\u0430', f2.GetName()[2]);
				AssertEquals('\u0431', f2.GetName()[4]);
				AssertEquals('\u0448', f2.GetName()[6]);
				AssertEquals('\u0429', f2.GetName()[8]);
			}
			finally
			{
				NeoDatis.Odb.OdbConfiguration.SetDatabaseCharacterEncoding(currentEncoding);
				if (server != null)
				{
					server.Close();
				}
			}
		}
	}
}
