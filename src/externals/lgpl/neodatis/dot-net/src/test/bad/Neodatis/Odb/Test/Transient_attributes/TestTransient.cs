namespace NeoDatis.Odb.Test.Transient_attributes
{
	public class TestTransient : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.Transient_attributes.VoWithTransientAttribute vo = new NeoDatis.Odb.Test.Transient_attributes.VoWithTransientAttribute
				("vo1");
			odb.Store(vo);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.Transient_attributes.VoWithTransientAttribute
				> vos = odb.GetObjects(typeof(NeoDatis.Odb.Test.Transient_attributes.VoWithTransientAttribute
				));
			odb.Close();
			Println(vos.GetFirst().GetName());
			AssertEquals(1, vos.Count);
			AssertEquals("vo1", vos.GetFirst().GetName());
		}
	}
}
