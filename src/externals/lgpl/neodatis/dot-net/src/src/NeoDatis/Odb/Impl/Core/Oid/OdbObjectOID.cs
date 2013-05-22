using System;
namespace NeoDatis.Odb.Impl.Core.Oid
{
	[System.Serializable]
	public class OdbObjectOID : NeoDatis.Odb.OID
	{
		protected long oid;
        private static Type OID_TYPE = typeof(OdbObjectOID);

		public OdbObjectOID(long oid)
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
				.TypeNameObjectOid).Append(":").Append(oid.ToString());
			return buffer.ToString();
		}

		public static NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID OidFromString(string oidString
			)
		{
			string[] tokens = NeoDatis.Tool.Wrappers.OdbString.Split(oidString, ":");
			if (tokens.Length != 2 || !(tokens[0].Equals(NeoDatis.Odb.OIDTypes.TypeNameObjectOid
				)))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InvalidOidRepresentation
					.AddParameter(oidString));
			}
			long oid = long.Parse(tokens[1]);
			return new NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID(oid);
		}

		public virtual long GetObjectId()
		{
			return oid;
		}

		public virtual int CompareTo(object o)
		{
            if (o == null || o.GetType() != OID_TYPE)
			{
				return -1000;
			}
            OdbObjectOID otherOid = (OdbObjectOID)o;
            int r = (int) (oid - otherOid.oid);
            return r; ;
		}

		public override bool Equals(object o)
		{
            if (this == o)
            {
                return true;
            }
            if (o == null || o.GetType() != OID_TYPE)
            {
                return false;
            }
            OdbObjectOID otherOid = (OdbObjectOID)o;
            return otherOid.oid == this.oid;
		}

		public override int GetHashCode()
		{
			//Copy of the Long hashcode algorithm  
			//return (int)(oid ^ ((oid) >> (32 & 0x1f)));
            return (int)(oid ^ (URShift(oid, 32)));
		}
        public long URShift(long number, int bits)
        {
            if (number >= 0)
                return number >> bits;
            else
                return (number >> bits) + (2L << ~bits);
        }
		public virtual long GetClassId()
		{
			return 0;
		}

		public virtual int GetType()
		{
			return NeoDatis.Odb.OIDTypes.TypeObjectOid;
		}
	}
}
