namespace NeoDatis.Odb.Impl.Core.Oid
{
	[System.Serializable]
	public class ExternalClassOID : NeoDatis.Odb.Impl.Core.Oid.OdbClassOID, NeoDatis.Odb.ExternalOID
	{
		private NeoDatis.Odb.DatabaseId databaseId;

		public ExternalClassOID(NeoDatis.Odb.OID oid, NeoDatis.Odb.DatabaseId databaseId)
			 : base(oid.GetObjectId())
		{
			this.databaseId = databaseId;
		}

		public virtual NeoDatis.Odb.DatabaseId GetDatabaseId()
		{
			return databaseId;
		}

		public override string OidToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder(NeoDatis.Odb.OIDTypes
				.TypeNameExternalClassOid).Append(":");
			buffer.Append(databaseId.ToString()).Append(":").Append(oid);
			return buffer.ToString();
		}

		public static NeoDatis.Odb.Impl.Core.Oid.ExternalClassOID OidFromString(string oidString
			)
		{
			string[] tokens = NeoDatis.Tool.Wrappers.OdbString.Split(oidString, ":");
			if (tokens.Length != 3 || !(tokens[0].Equals(NeoDatis.Odb.OIDTypes.TypeNameExternalClassOid
				)))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InvalidOidRepresentation
					.AddParameter(oidString));
			}
			long oid = long.Parse(tokens[2]);
			string databaseid = tokens[1];
			return new NeoDatis.Odb.Impl.Core.Oid.ExternalClassOID(new NeoDatis.Odb.Impl.Core.Oid.OdbClassOID
				(oid), NeoDatis.Odb.Impl.Core.Oid.DatabaseIdImpl.FromString(databaseid));
		}

		public override int GetType()
		{
			return NeoDatis.Odb.OIDTypes.TypeExternalClassOid;
		}
	}
}
