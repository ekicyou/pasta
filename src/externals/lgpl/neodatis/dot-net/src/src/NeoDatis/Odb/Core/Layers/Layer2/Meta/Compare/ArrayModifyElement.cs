namespace NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare
{
	public class ArrayModifyElement
	{
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi;

		/// <summary>The array id</summary>
		private int attributeId;

		private int arrayElementIndexToChange;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo newValue;

		private bool supportInPlaceUpdate;

		public virtual int GetAttributeId()
		{
			return attributeId;
		}

		public virtual void SetAttributeId(int attributeId)
		{
			this.attributeId = attributeId;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetNnoi()
		{
			return nnoi;
		}

		public virtual void SetNnoi(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			this.nnoi = nnoi;
		}

		public ArrayModifyElement(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, int attributeId, int index, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 newValue, bool supportInPlaceUpdate) : base()
		{
			this.nnoi = nnoi;
			this.attributeId = attributeId;
			this.supportInPlaceUpdate = supportInPlaceUpdate;
			this.newValue = newValue;
			this.arrayElementIndexToChange = index;
		}

		public virtual long GetUpdatePosition()
		{
			return nnoi.GetAttributeDefinitionPosition(attributeId);
		}

		public virtual int GetArrayElementIndexToChange()
		{
			return arrayElementIndexToChange;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo GetNewValue
			()
		{
			return newValue;
		}

		/// <summary>Return the position where the array position is stored</summary>
		/// <returns></returns>
		public virtual long GetArrayPositionDefinition()
		{
			return nnoi.GetAttributeDefinitionPosition(attributeId);
		}

		public virtual bool SupportInPlaceUpdate()
		{
			return supportInPlaceUpdate;
		}
	}
}
