using NeoDatis.Odb.Test.VO.Login;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Update
{
	[TestFixture]
    public class TestUnconnectedZone : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			DeleteBase("unconnected");
			NeoDatis.Odb.ODB odb = Open("unconnected");
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f1"));
			odb.Close();
			Println("Oid=" + oid);
			odb = Open("unconnected");
			NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)odb
				.GetObjectFromId(oid);
			f2.SetName("New Function");
			odb.Store(f2);
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(odb);
			// retrieve the class info to check connected and unconnected zone
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = storageEngine.GetSession(true
				).GetMetaModel().GetClassInfo(typeof(NeoDatis.Odb.Test.VO.Login.Function).FullName
				, true);
			odb.Close();
			AssertEquals(1, ci.GetCommitedZoneInfo().GetNbObjects());
			AssertNotNull(ci.GetCommitedZoneInfo().first);
			AssertNotNull(ci.GetCommitedZoneInfo().last);
			AssertEquals(0, ci.GetUncommittedZoneInfo().GetNbObjects());
			AssertNull(ci.GetUncommittedZoneInfo().first);
			AssertNull(ci.GetUncommittedZoneInfo().last);
			odb = Open("unconnected");
			AssertEquals(1, odb.GetObjects<Function>().Count
				);
			odb.Close();
		}
	}
}
