using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	[TestFixture]
    public class TestCriteriaQuery6 : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			System.Collections.Generic.IList<Profile> profiles = new 
				System.Collections.Generic.List<Profile>();
			profiles.Add(new Profile("p1", new Function
				("f1")));
			profiles.Add(new Profile("p2", new Function
				("f2")));
			ClassB cb = new ClassB
				("name", profiles);
			odb.Store(cb);
			odb.Close();
			odb = Open(baseName);
			// this object is not known y NeoDatis so the query will not return anything
			Profile p = new Profile("p1", (System.Collections.Generic.IList<Function>)null);
			CriteriaQuery query = odb.CriteriaQuery(typeof(
				ClassB), Where
				.Contain("profiles", p));
			NeoDatis.Odb.Objects<ClassB> l = odb.GetObjects<ClassB>(
				query);
			odb.Close();
			AssertEquals(0, l.Count);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			System.Collections.Generic.IList<Profile> profiles = new 
				System.Collections.Generic.List<Profile>();
			profiles.Add(new Profile("p1", new Function("f1")));
			profiles.Add(new Profile("p2", new Function("f2")));
			ClassB cb = new ClassB("name", profiles);
			odb.Store(cb);
			odb.Close();
			odb = Open(baseName);
			Profile p = (Profile)odb.GetObjects<Profile>().GetFirst();
			CriteriaQuery query = odb.CriteriaQuery(typeof(ClassB), Where
				.Contain("profiles", p));
			NeoDatis.Odb.Objects<ClassB> l = odb.GetObjects<ClassB>(
				query);
			odb.Close();
			AssertEquals(1, l.Count);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestReuse()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			System.Collections.Generic.IList<Profile> profiles = new 
				System.Collections.Generic.List<Profile>();
			profiles.Add(new Profile("p1", new Function
				("f1")));
			profiles.Add(new Profile("p2", new Function
				("f2")));
			ClassB cb = new ClassB
				("name", profiles);
			odb.Store(cb);
			odb.Close();
			odb = Open(baseName);
			Profile p = (Profile)odb.GetObjects<Profile>().GetFirst();
			CriteriaQuery query = odb.CriteriaQuery(typeof(
				ClassB), Where.Equal("profiles", p));
			EqualCriterion ec = (EqualCriterion	)query.GetCriteria();
			try
			{
				NeoDatis.Odb.Objects<ClassB> l = odb.GetObjects<ClassB>(
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
