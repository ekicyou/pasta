using NUnit.Framework;
namespace NeoDatis.Odb.Test.Transient_attributes
{
	[TestFixture]
    public class TestTransient : NeoDatis.Odb.Test.ODBTest
	{
		[Test]
        public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.Transient_attributes.VoWithTransientAttribute vo = new NeoDatis.Odb.Test.Transient_attributes.VoWithTransientAttribute
				("vo1");
			odb.Store(vo);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects<VoWithTransientAttribute> vos = odb.GetObjects<VoWithTransientAttribute>();
			odb.Close();
			Println(vos.GetFirst().GetName());
			AssertEquals(1, vos.Count);
			AssertEquals("vo1", vos.GetFirst().GetName());
		}
	}
}
