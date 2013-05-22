namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>To keep info about a native instance</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public abstract class NativeObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
	{
		/// <summary>The object being represented</summary>
		protected object theObject;

		public NativeObjectInfo(object @object, int odbTypeId) : base(odbTypeId)
		{
			this.theObject = @object;
		}

		public NativeObjectInfo(object @object, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			 odbType) : base(odbType)
		{
			this.theObject = @object;
		}

		public override string ToString()
		{
			if (theObject != null)
			{
				return theObject.ToString();
			}
			return "null";
		}

		public override bool Equals(object obj)
		{
			if (obj == null)
			{
				return false;
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo noi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
				)obj;
			if (theObject == noi.GetObject())
			{
				return true;
			}
			return theObject.Equals(noi.GetObject());
		}

		public virtual bool IsNativeObject()
		{
			return true;
		}

		public override object GetObject()
		{
			return theObject;
		}

		public override void SetObject(object @object)
		{
			this.theObject = @object;
		}
	}
}
