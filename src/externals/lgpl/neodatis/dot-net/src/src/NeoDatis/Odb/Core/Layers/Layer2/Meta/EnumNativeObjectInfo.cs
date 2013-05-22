namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>Meta representation of an enum.</summary>
	/// <remarks>
	/// Meta representation of an enum. Which is internally represented by a string :
	/// Its name
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class EnumNativeObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
	{
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo enumClassInfo;

		public EnumNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			, string enumName) : base(enumName, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.EnumId)
		{
			this.enumClassInfo = classInfo;
		}

		public override string ToString()
		{
			return GetObject().ToString();
		}

		public override bool IsNull()
		{
			return GetObject() == null;
		}

		public override bool IsNative()
		{
			return true;
		}

		public override bool IsEnumObject()
		{
			return true;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo CreateCopy
			(System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> cache, bool onlyData)
		{
			return new NeoDatis.Odb.Core.Layers.Layer2.Meta.EnumNativeObjectInfo(enumClassInfo
				, GetObject() == null ? null : GetObject().ToString());
		}

		public virtual string GetEnumName()
		{
			return GetObject().ToString();
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo GetEnumClassInfo()
		{
			return enumClassInfo;
		}

		public virtual void SetEnumClassInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 enumClassInfo)
		{
			this.enumClassInfo = enumClassInfo;
		}
	}
}
