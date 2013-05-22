using NUnit.Framework;
namespace NeoDatis.Odb.Test.Oid
{
	[TestFixture]
    public class TestOidAsNativeObject : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			NeoDatis.Odb.Test.Oid.ClassWithOid cwo = new NeoDatis.Odb.Test.Oid.ClassWithOid("test"
				, NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(47));
			DeleteBase("native-oid");
			NeoDatis.Odb.ODB odb = Open("native-oid");
			odb.Store(cwo);
			odb.Close();
			odb = Open("native-oid");
			NeoDatis.Odb.Objects<ClassWithOid> objects = odb.GetObjects<ClassWithOid>();
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.Oid.ClassWithOid cwo2 = (NeoDatis.Odb.Test.Oid.ClassWithOid)objects
				.GetFirst();
			AssertEquals(47, cwo2.GetOid().GetObjectId());
		}
	}
}
