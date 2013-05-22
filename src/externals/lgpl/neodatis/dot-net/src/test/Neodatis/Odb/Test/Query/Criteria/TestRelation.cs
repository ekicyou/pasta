using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	[TestFixture]
    public class TestRelation : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestNullRelation()
		{
			DeleteBase("null-rel.neodatis");
			NeoDatis.Odb.ODB odb = Open("null-rel.neodatis");
			odb.Store(new NeoDatis.Odb.Test.Query.Criteria.Class2());
			odb.Close();
			odb = Open("null-rel.neodatis");
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Criteria.Class2), NeoDatis.Odb.Core.Query.Criteria.Where
				.IsNull("class1.name"));
			NeoDatis.Odb.Objects<Class2> os = odb.GetObjects<Class2>(q);
			odb.Close();
			AssertEquals(1, os.Count);
			NeoDatis.Odb.Test.Query.Criteria.Class2 c2 = (NeoDatis.Odb.Test.Query.Criteria.Class2
				)os.GetFirst();
			AssertEquals(null, c2.GetClass1());
		}
	}
}
