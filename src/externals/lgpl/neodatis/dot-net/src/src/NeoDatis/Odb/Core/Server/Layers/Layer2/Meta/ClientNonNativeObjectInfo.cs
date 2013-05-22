namespace NeoDatis.Odb.Core.Server.Layers.Layer2.Meta
{
	[System.Serializable]
	public class ClientNonNativeObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
	{
		protected NeoDatis.Odb.OID localOid;

		public ClientNonNativeObjectInfo()
		{
		}

		public ClientNonNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 oip, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo) : base(oip, classInfo
			)
		{
		}

		public ClientNonNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			) : base(classInfo)
		{
		}

		public ClientNonNativeObjectInfo(object @object, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 info, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo[] values, long[] 
			attributesIdentification, int[] attributeIds) : base(@object, info, values, attributesIdentification
			, attributeIds)
		{
		}

		public virtual NeoDatis.Odb.OID GetLocalOid()
		{
			return localOid;
		}

		public virtual void SetLocalOid(NeoDatis.Odb.OID localOid)
		{
			this.localOid = localOid;
		}
	}
}
