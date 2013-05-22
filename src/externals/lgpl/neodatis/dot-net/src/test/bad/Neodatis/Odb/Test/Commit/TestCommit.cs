namespace NeoDatis.Odb.Test.Commit
{
	public class TestCommit : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithCommitsSimpleObject()
		{
			DeleteBase("commits");
			NeoDatis.Odb.ODB odb = null;
			int size = isLocal ? 10000 : 3000;
			int commitInterval = 1;
			try
			{
				odb = Open("commits");
				for (int i = 0; i < size; i++)
				{
					odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function " + i));
					if (i % commitInterval == 0)
					{
						odb.Commit();
					}
				}
			}
			finally
			{
				// println("commiting "+i);
				odb.Close();
			}
			odb = Open("commits");
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			int nbObjects = objects.Count;
			System.Collections.IDictionary map = new NeoDatis.Tool.Wrappers.Map.OdbHashMap();
			NeoDatis.Odb.Test.VO.Login.Function function = null;
			int j = 0;
			while (objects.HasNext())
			{
				function = (NeoDatis.Odb.Test.VO.Login.Function)objects.Next();
				int ii = (int)map[function];
				if (ii != null)
				{
					Println(j + ":" + function.GetName() + " already exist at " + ii);
				}
				else
				{
					map.Add(function, j);
				}
				j++;
			}
			odb.Close();
			DeleteBase("commits");
			Println("Nb objects=" + nbObjects);
			AssertEquals(size, nbObjects);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertWithCommitsComplexObject()
		{
			DeleteBase("commits");
			NeoDatis.Odb.ODB odb = null;
			int size = isLocal ? 5300 : 500;
			int commitInterval = 400;
			try
			{
				odb = Open("commits");
				for (int i = 0; i < size; i++)
				{
					odb.Store(GetInstance(i));
					if (i % commitInterval == 0)
					{
						odb.Commit();
					}
					// println("commiting "+i);
					if (i % 100 == 0 && !isLocal)
					{
						Println(i);
					}
				}
			}
			finally
			{
				odb.Close();
			}
			odb = Open("commits");
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				));
			NeoDatis.Odb.Objects profiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile
				));
			NeoDatis.Odb.Objects functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			int nbUsers = users.Count;
			int nbProfiles = profiles.Count;
			int nbFunctions = functions.Count;
			odb.Close();
			DeleteBase("commits");
			Println("Nb users=" + nbUsers);
			Println("Nb profiles=" + nbProfiles);
			Println("Nb functions=" + nbFunctions);
			AssertEquals(size, nbUsers);
			AssertEquals(size, nbProfiles);
			AssertEquals(size * 2, nbFunctions);
		}

		private object GetInstance(int i)
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
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier"
				 + i, "olivier@neodatis.com", profile);
			return user;
		}
	}
}
