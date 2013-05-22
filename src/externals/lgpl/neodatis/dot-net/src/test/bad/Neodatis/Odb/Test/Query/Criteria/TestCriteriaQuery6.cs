namespace NeoDatis.Odb.Test.Query.Criteria
{
	public class TestCriteriaQuery6 : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Login.Profile> profiles = new 
				System.Collections.Generic.List<NeoDatis.Odb.Test.VO.Login.Profile>();
			profiles.Add(new NeoDatis.Odb.Test.VO.Login.Profile("p1", new NeoDatis.Odb.Test.VO.Login.Function
				("f1")));
			profiles.Add(new NeoDatis.Odb.Test.VO.Login.Profile("p2", new NeoDatis.Odb.Test.VO.Login.Function
				("f2")));
			NeoDatis.Odb.Test.Query.Criteria.ClassB cb = new NeoDatis.Odb.Test.Query.Criteria.ClassB
				("name", profiles);
			odb.Store(cb);
			odb.Close();
			odb = Open(baseName);
			// this object is not known y NeoDatis so the query will not return anything
			NeoDatis.Odb.Test.VO.Login.Profile p = new NeoDatis.Odb.Test.VO.Login.Profile("p1"
				, (System.Collections.IList)null);
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query = odb.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.Query.Criteria.ClassB), NeoDatis.Odb.Core.Query.Criteria.Where
				.Contain("profiles", p));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Query.Criteria.ClassB> l = odb.GetObjects(
				query);
			odb.Close();
			AssertEquals(0, l.Count);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Login.Profile> profiles = new 
				System.Collections.Generic.List<NeoDatis.Odb.Test.VO.Login.Profile>();
			profiles.Add(new NeoDatis.Odb.Test.VO.Login.Profile("p1", new NeoDatis.Odb.Test.VO.Login.Function
				("f1")));
			profiles.Add(new NeoDatis.Odb.Test.VO.Login.Profile("p2", new NeoDatis.Odb.Test.VO.Login.Function
				("f2")));
			NeoDatis.Odb.Test.Query.Criteria.ClassB cb = new NeoDatis.Odb.Test.Query.Criteria.ClassB
				("name", profiles);
			odb.Store(cb);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Profile p = (NeoDatis.Odb.Test.VO.Login.Profile)odb.GetObjects
				(typeof(NeoDatis.Odb.Test.VO.Login.Profile)).GetFirst();
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query = odb.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.Query.Criteria.ClassB), NeoDatis.Odb.Core.Query.Criteria.Where
				.Contain("profiles", p));
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Query.Criteria.ClassB> l = odb.GetObjects(
				query);
			odb.Close();
			AssertEquals(1, l.Count);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestReuse()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Login.Profile> profiles = new 
				System.Collections.Generic.List<NeoDatis.Odb.Test.VO.Login.Profile>();
			profiles.Add(new NeoDatis.Odb.Test.VO.Login.Profile("p1", new NeoDatis.Odb.Test.VO.Login.Function
				("f1")));
			profiles.Add(new NeoDatis.Odb.Test.VO.Login.Profile("p2", new NeoDatis.Odb.Test.VO.Login.Function
				("f2")));
			NeoDatis.Odb.Test.Query.Criteria.ClassB cb = new NeoDatis.Odb.Test.Query.Criteria.ClassB
				("name", profiles);
			odb.Store(cb);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Profile p = (NeoDatis.Odb.Test.VO.Login.Profile)odb.GetObjects
				(typeof(NeoDatis.Odb.Test.VO.Login.Profile)).GetFirst();
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query = odb.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.Query.Criteria.ClassB), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("profiles", p));
			NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion ec = (NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion
				)query.GetCriteria();
			try
			{
				NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Query.Criteria.ClassB> l = odb.GetObjects(
					query);
			}
			catch (System.Exception e)
			{
				AssertTrue(NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, true).IndexOf("1063"
					) != -1);
			}
			odb.Close();
		}
	}
}
