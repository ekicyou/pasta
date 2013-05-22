namespace NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare
{
	public class SetAttributeToNullAction
	{
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi;

		private int attributeId;

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

		public SetAttributeToNullAction(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, int attributeId) : base()
		{
			this.nnoi = nnoi;
			this.attributeId = attributeId;
		}

		public virtual long GetUpdatePosition()
		{
			return nnoi.GetAttributeDefinitionPosition(attributeId);
		}
	}
}
