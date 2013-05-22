namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>A super class for CollectionObjectInfo, MapObjectInfo and ArrayObjectInfo.
	/// 	</summary>
	/// <remarks>
	/// A super class for CollectionObjectInfo, MapObjectInfo and ArrayObjectInfo. It keeps a list
	/// of reference to non native objects contained in theses structures
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public abstract class GroupObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
	{
		private System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> nonNativeObjects;

		public GroupObjectInfo(object @object, int odbTypeId) : base(@object, odbTypeId)
		{
			this.nonNativeObjects = new System.Collections.Generic.List<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				>();
		}

		public GroupObjectInfo(object @object, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			 odbType) : base(@object, odbType)
		{
			this.nonNativeObjects = new System.Collections.Generic.List<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				>();
		}

		public virtual System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> GetNonNativeObjects()
		{
			return nonNativeObjects;
		}

		public virtual void SetNonNativeObjects(System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> nonNativeObjects)
		{
			this.nonNativeObjects = nonNativeObjects;
		}

		public virtual void AddNonNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			this.nonNativeObjects.Add(nnoi);
		}
	}
}
