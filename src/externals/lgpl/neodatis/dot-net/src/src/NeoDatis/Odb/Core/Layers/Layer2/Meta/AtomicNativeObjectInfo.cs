namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>To keep info about a native object like int,char, long, Does not include array or collection
	/// 	</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class AtomicNativeObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo
		, System.IComparable
	{
		public AtomicNativeObjectInfo(object @object, int odbTypeId) : base(@object, odbTypeId
			)
		{
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
			if (obj == null || !(obj is NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				))
			{
				return false;
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo noi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				)obj;
			if (theObject == noi.GetObject())
			{
				return true;
			}
			return theObject.Equals(noi.GetObject());
		}

		public override bool IsAtomicNativeObject()
		{
			return true;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo CreateCopy
			(System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> cache, bool onlyData)
		{
			return new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(theObject, 
				odbTypeId);
		}

		public virtual int CompareTo(object o)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				)o;
			System.IComparable c2 = (System.IComparable)anoi.GetObject();
			System.IComparable c1 = (System.IComparable)theObject;
			return c1.CompareTo(c2);
		}
	}
}
