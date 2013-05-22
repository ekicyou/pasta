namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>To keep info about a non native null instance</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class NonNativeNullObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
	{
		public NonNativeNullObjectInfo() : base(null)
		{
		}

		public NonNativeNullObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			) : base(classInfo)
		{
		}

		public override string ToString()
		{
			return "null non native object ";
		}

		public virtual bool HasChanged(NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 aoi)
		{
			return aoi.GetType() != typeof(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeNullObjectInfo
				);
		}

		public virtual bool IsNonNativeNullObject()
		{
			return true;
		}

		public override bool IsNull()
		{
			return true;
		}
	}
}
