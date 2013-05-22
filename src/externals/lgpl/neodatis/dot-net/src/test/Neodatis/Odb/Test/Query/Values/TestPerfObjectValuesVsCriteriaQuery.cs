using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Values
{
	[TestFixture]
    public class TestPerfObjectValuesVsCriteriaQuery : NeoDatis.Odb.Test.ODBTest
	{
		[Test]
        public virtual void Test()
		{
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Populate()
		{
			NeoDatis.Odb.ODB odb = Open("perfOValuesVsCriteria");
			int nbProfiles = 200;
			int nbUsers = 500000;
			NeoDatis.Odb.Test.VO.Login.Profile[] profiles = new NeoDatis.Odb.Test.VO.Login.Profile
				[nbProfiles];
			NeoDatis.Odb.Test.VO.Login.User2[] users = new NeoDatis.Odb.Test.VO.Login.User2[nbUsers
				];
			// First creates profiles
			for (int i = 0; i < nbProfiles; i++)
			{
				profiles[i] = new NeoDatis.Odb.Test.VO.Login.Profile("profile " + i, new NeoDatis.Odb.Test.VO.Login.Function
					("function Profile" + i));
				odb.Store(profiles[i]);
			}
			// Then creates users
			for (int i = 0; i < nbUsers; i++)
			{
				users[i] = new NeoDatis.Odb.Test.VO.Login.User2("user" + i, "user mail" + i, profiles
					[GetProfileIndex(nbProfiles)], i);
				odb.Store(users[i]);
				if (i % 10000 == 0)
				{
					Println(i);
				}
			}
			odb.Close();
		}

		private int GetProfileIndex(int nbProfiles)
		{
			return (int)System.Math.Random() * nbProfiles;
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Query.Values.TestPerfObjectValuesVsCriteriaQuery t = new NeoDatis.Odb.Test.Query.Values.TestPerfObjectValuesVsCriteriaQuery
				();
			// t.populate();
			t.T1estA();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1est()
		{
			NeoDatis.Odb.ODB odb = Open("perfOValuesVsCriteria");
			NeoDatis.Odb.OdbConfiguration.MonitorMemory(true);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User2));
			System.Decimal b = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User2)));
			Println(b);
			System.Console.Out.WriteLine(q.GetExecutionPlan().GetDetails());
			AssertEquals(new System.Decimal("500000"), b);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estA()
		{
			NeoDatis.Odb.ODB odb = Open("perfOValuesVsCriteria");
			NeoDatis.Odb.OdbConfiguration.MonitorMemory(true);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User2));
			NeoDatis.Odb.Objects objects = odb.GetObjects(q, false);
			Println(objects.Count);
			System.Console.Out.WriteLine(q.GetExecutionPlan().GetDetails());
			AssertEquals(2000000, objects.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1est1()
		{
			NeoDatis.Odb.ODB odb = Open("perfOValuesVsCriteria");
			NeoDatis.Odb.OdbConfiguration.MonitorMemory(true);
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User2), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("nbLogins", 100)).Field("name");
			NeoDatis.Odb.Values v = odb.GetValues(q);
			Println(v.Count);
			System.Console.Out.WriteLine(q.GetExecutionPlan().GetDetails());
			AssertEquals(2000000, v.Count);
			odb.Close();
		}
	}
}
