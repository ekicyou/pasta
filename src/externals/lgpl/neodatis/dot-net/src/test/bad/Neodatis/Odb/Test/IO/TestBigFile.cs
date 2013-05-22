using System.IO;
namespace NeoDatis.Odb.Test.IO
{
	public class TestBigFile : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.IO.IOException"></exception>
		public virtual void Test1()
		{
			System.IO.FileStream raf = new System.IO.FileStream(NeoDatis.Odb.Test.ODBTest.Directory
				 + "testBigFile", FileMode.OpenOrCreate);
			long l = 2 * 1024000;
			Println(l);
			raf.Seek(l);
			for (int i = 0; i < 1024000; i++)
			{
				raf.Write((byte)0);
			}
			raf.Write((byte)0);
			raf.Close();
		}

		private object GetUserInstance(int i)
		{
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login" + i);
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout" + i);
			System.Collections.IList list = new System.Collections.ArrayList();
			list.Add(login);
			list.Add(logout);
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator" + i, list);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				 + i, "olivier@neodatis.com", profile);
			return user;
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void T2estBigFileWithOdb()
		{
			int size1 = 10000;
			int size2 = 1000;
			string baseName = "big-file.neodatis";
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(baseName);
				odb.Close();
				int z = 0;
				for (int i = 0; i < size1; i++)
				{
					odb = Open(baseName);
					for (int j = 0; j < size2; j++)
					{
						odb.Store(GetUserInstance(j));
						z++;
					}
					odb.Close();
					Println(i + "/" + size1 + " " + z + " objects");
				}
			}
			finally
			{
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		public virtual void T2estBigFileWithOdbSelect()
		{
			// OdbConfiguration.setUseIndex(false);
			string baseName = "big-file.neodatis";
			NeoDatis.Odb.ODB odb = null;
			// Thread.sleep(20000);
			try
			{
				long start = Sharpen.Runtime.CurrentTimeMillis();
				odb = Open(baseName);
				NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "login10000"));
				NeoDatis.Odb.Objects<NeoDatis.Odb.Test.VO.Login.Function> functions = odb.GetObjects
					(q, true, 0, 1);
				System.Console.Out.WriteLine(q.GetExecutionPlan().GetDetails());
				System.Console.Out.WriteLine(functions.Count);
				Println(Sharpen.Runtime.CurrentTimeMillis() - start + "ms");
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.IO.TestBigFile tt = new NeoDatis.Odb.Test.IO.TestBigFile();
			// tt.t2estBigFileWithOdbSelect();
			tt.T2estBigFileWithOdb();
		}
	}
}
