namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>To specify that an object has been mark as deleted</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class NonNativeDeletedObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
	{
		private NeoDatis.Odb.OID oid;

		public NonNativeDeletedObjectInfo(long position, NeoDatis.Odb.OID oid) : base(null
			, null)
		{
			this.position = position;
			this.oid = oid;
		}

		public override string ToString()
		{
			return "deleted";
		}

		public virtual bool HasChanged(NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 aoi)
		{
			return aoi.GetType() != typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeDeletedObjectInfo
				);
		}

		public override object GetObject()
		{
			return null;
		}

		public override bool IsDeletedObject()
		{
			return true;
		}

		/// <summary>A deleted non native object is considered to be null!</summary>
		public override bool IsNull()
		{
			return true;
		}
	}
}
