namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>Meta representation of a null native object</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class NullNativeObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
	{
		public static NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo instance = 
			new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo();

		private NullNativeObjectInfo() : base(null, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.Null)
		{
		}

		public NullNativeObjectInfo(int odbTypeId) : base(null, odbTypeId)
		{
		}

		public NullNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type) : 
			base(null, type)
		{
		}

		public override string ToString()
		{
			return "null";
		}

		public override bool IsNull()
		{
			return true;
		}

		public override bool IsNative()
		{
			return true;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo CreateCopy
			(System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> cache, bool onlyData)
		{
			return NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo.GetInstance();
		}

		/// <returns></returns>
		public static NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo GetInstance
			()
		{
			return instance;
		}
	}
}
