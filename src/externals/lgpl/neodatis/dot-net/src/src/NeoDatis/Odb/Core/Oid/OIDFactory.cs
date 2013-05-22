namespace NeoDatis.Odb.Core.Oid
{
	public class OIDFactory
	{
		public static NeoDatis.Odb.OID BuildObjectOID(long oid)
		{
			return NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetObjectOID(oid, 0);
		}

		public static NeoDatis.Odb.OID BuildClassOID(long oid)
		{
			return NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassOID(oid);
		}

		public static NeoDatis.Odb.OID OidFromString(string oidString)
		{
			string[] tokens = NeoDatis.Tool.Wrappers.OdbString.Split(oidString, ":");
			if (tokens[0].Equals(NeoDatis.Odb.OIDTypes.TypeNameObjectOid))
			{
				return NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID.OidFromString(oidString);
			}
			if (tokens[0].Equals(NeoDatis.Odb.OIDTypes.TypeNameClassOid))
			{
				return NeoDatis.Odb.Impl.Core.Oid.OdbClassOID.OidFromString(oidString);
			}
			if (tokens[0].Equals(NeoDatis.Odb.OIDTypes.TypeNameExternalObjectOid))
			{
				return NeoDatis.Odb.Impl.Core.Oid.ExternalObjectOID.OidFromString(oidString);
			}
			if (tokens[0].Equals(NeoDatis.Odb.OIDTypes.TypeNameExternalClassOid))
			{
				return NeoDatis.Odb.Impl.Core.Oid.ExternalClassOID.OidFromString(oidString);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InvalidOidRepresentation
				.AddParameter(oidString));
		}
	}
}
