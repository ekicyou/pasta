namespace NeoDatis.Odb.Impl.Core.Oid
{
	[System.Serializable]
	public class OdbClassOID : NeoDatis.Odb.OID
	{
		protected long oid;

		public OdbClassOID(long oid)
		{
			this.oid = oid;
		}

		public override string ToString()
		{
			return oid.ToString();
		}

		public virtual string OidToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder(NeoDatis.Odb.OIDTypes
				.TypeNameClassOid).Append(":").Append(oid.ToString());
			return buffer.ToString();
		}

		public static NeoDatis.Odb.Impl.Core.Oid.OdbClassOID OidFromString(string oidString
			)
		{
			string[] tokens = NeoDatis.Tool.Wrappers.OdbString.Split(oidString, ":");
			if (tokens.Length != 2 || !(tokens[0].Equals(NeoDatis.Odb.OIDTypes.TypeNameClassOid
				)))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InvalidOidRepresentation
					.AddParameter(oidString));
			}
			long oid = long.Parse(tokens[1]);
			return new NeoDatis.Odb.Impl.Core.Oid.OdbClassOID(oid);
		}

		public virtual long GetObjectId()
		{
			return oid;
		}

		public virtual int CompareTo(object @object)
		{
			if (@object == null || !(@object is NeoDatis.Odb.Impl.Core.Oid.OdbClassOID))
			{
				return -1000;
			}
			NeoDatis.Odb.OID otherOid = (NeoDatis.Odb.OID)@object;
			return (int)(oid - otherOid.GetObjectId());
		}

		public override bool Equals(object @object)
		{
			bool b = this == @object || this.CompareTo(@object) == 0;
			return b;
		}

		public override int GetHashCode()
		{
			// Copy of the Long hashcode algorithm
			return (int)(oid ^ ((oid) >> (32 & 0x1f)));
		}

		public virtual long GetClassId()
		{
			return 0;
		}

		public virtual int GetType()
		{
			return NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ClassOidId;
		}
	}
}
