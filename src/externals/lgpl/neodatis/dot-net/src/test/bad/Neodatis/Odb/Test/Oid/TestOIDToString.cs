namespace NeoDatis.Odb.Test.Oid
{
	/// <author>olivier</author>
	public class TestOIDToString : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("My Function"
				));
			odb.Close();
			NeoDatis.Tool.IOUtil.DeleteFile(baseName);
			string soid = oid.OidToString();
			NeoDatis.Odb.OID oid2 = NeoDatis.Odb.Core.Oid.OIDFactory.OidFromString(soid);
			AssertEquals(oid, oid2);
		}

		public virtual void Test3()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.OID oid = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("My Function"
				));
			oid = odb.Ext().ConvertToExternalOID(oid);
			odb.Close();
			NeoDatis.Tool.IOUtil.DeleteFile(baseName);
			string soid = oid.OidToString();
			Println(soid);
			NeoDatis.Odb.OID oid2 = NeoDatis.Odb.Core.Oid.OIDFactory.OidFromString(soid);
			AssertEquals(oid, oid2);
		}

		public virtual void Test2()
		{
			NeoDatis.Odb.Impl.Core.Oid.OdbClassOID oid = new NeoDatis.Odb.Impl.Core.Oid.OdbClassOID
				(10002);
			string soid = oid.OidToString();
			NeoDatis.Odb.OID oid2 = NeoDatis.Odb.Core.Oid.OIDFactory.OidFromString(soid);
			AssertEquals(oid, oid2);
		}

		public virtual void Test4()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Impl.Core.Oid.ExternalClassOID oid = new NeoDatis.Odb.Impl.Core.Oid.ExternalClassOID
				(new NeoDatis.Odb.Impl.Core.Oid.OdbClassOID(19), odb.Ext().GetDatabaseId());
			string soid = oid.OidToString();
			NeoDatis.Odb.OID oid2 = NeoDatis.Odb.Core.Oid.OIDFactory.OidFromString(soid);
			AssertEquals(oid, oid2);
		}
	}
}
