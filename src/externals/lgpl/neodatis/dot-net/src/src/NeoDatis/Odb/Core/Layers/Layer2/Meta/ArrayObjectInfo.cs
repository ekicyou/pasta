namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>A meta representation of an Array</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ArrayObjectInfo : GroupObjectInfo
	{
		private string realArrayComponentClassName;

		private int componentTypeId;

		public ArrayObjectInfo(object[] array) : base(array, ODBType.ArrayId)
		{
			realArrayComponentClassName = ODBType.DefaultArrayComponentClassName;
		}

		public ArrayObjectInfo(object[] array, ODBType type, int componentId) : base(array, type)
		{
			realArrayComponentClassName = ODBType.DefaultArrayComponentClassName;
			componentTypeId = componentId;
		}

		public virtual object[] GetArray()
		{
			return (object[])theObject;
		}

		public override string ToString()
		{
			if (theObject != null)
			{
				System.Text.StringBuilder buffer = new System.Text.StringBuilder();
				object[] array = GetArray();
				int length = array.Length;
				buffer.Append("[").Append(length).Append("]=(");
				for (int i = 0; i < length; i++)
				{
					if (i != 0)
					{
						buffer.Append(",");
					}
					buffer.Append(array[i]);
				}
				buffer.Append(")");
				return buffer.ToString();
			}
			return "null array";
		}

		public override bool IsArrayObject()
		{
			return true;
		}

		public virtual string GetRealArrayComponentClassName()
		{
			return realArrayComponentClassName;
		}

		public virtual void SetRealArrayComponentClassName(string realArrayComponentClassName
			)
		{
			this.realArrayComponentClassName = realArrayComponentClassName;
		}

		public virtual int GetArrayLength()
		{
			return GetArray().Length;
		}

		public virtual int GetComponentTypeId()
		{
			return componentTypeId;
		}

		public virtual void SetComponentTypeId(int componentTypeId)
		{
			this.componentTypeId = componentTypeId;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo CreateCopy
			(System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> cache, bool onlyData)
		{
			object[] array = GetArray();
			int length = array.Length;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo[] aois = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				[length];
			for (int i = 0; i < length; i++)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
					)array[i];
				aois[i] = aoi.CreateCopy(cache, onlyData);
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo arrayOfAoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ArrayObjectInfo
				(aois);
			arrayOfAoi.SetRealArrayComponentClassName(realArrayComponentClassName);
			arrayOfAoi.SetComponentTypeId(componentTypeId);
			return arrayOfAoi;
		}
	}
}
