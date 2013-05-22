namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>Meta representation of a collection</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class CollectionObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.GroupObjectInfo
	{
		private string realCollectionClassName;

		public CollectionObjectInfo() : base(null, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.CollectionId)
		{
			realCollectionClassName = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DefaultCollectionClassName;
		}

		public CollectionObjectInfo(System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> collection) : base(collection, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.CollectionId
			)
		{
			realCollectionClassName = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DefaultCollectionClassName;
		}

		public CollectionObjectInfo(System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> collection, System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> nonNativeObjects) : base(collection, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
			.CollectionId)
		{
			realCollectionClassName = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DefaultCollectionClassName;
			SetNonNativeObjects(nonNativeObjects);
		}

		public CollectionObjectInfo(System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> collection, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType type, System.Collections.Generic.ICollection
			<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo> nonNativeObjects) : base
			(collection, type)
		{
			realCollectionClassName = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DefaultCollectionClassName;
			SetNonNativeObjects(nonNativeObjects);
		}

		public virtual System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> GetCollection()
		{
			return (System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				>)theObject;
		}

		public override string ToString()
		{
			if (theObject != null)
			{
				return theObject.ToString();
			}
			return "null collection";
		}

		public override bool IsCollectionObject()
		{
			return true;
		}

		public virtual string GetRealCollectionClassName()
		{
			return realCollectionClassName;
		}

		public virtual void SetRealCollectionClassName(string realCollectionClass)
		{
			this.realCollectionClassName = realCollectionClass;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo CreateCopy
			(System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			> cache, bool onlyData)
		{
			System.Collections.ICollection c = (System.Collections.ICollection)theObject;
			System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				> newCollection = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				>();
			// To keep track of non native objects
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				> nonNatives = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				>();
			System.Collections.IEnumerator iterator = c.GetEnumerator();
			while (iterator.MoveNext())
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
					)iterator.Current;
				// create copy
				aoi = aoi.CreateCopy(cache, onlyData);
				newCollection.Add(aoi);
				if (aoi.IsNonNativeObject())
				{
					nonNatives.Add((NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)aoi);
				}
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo coi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.CollectionObjectInfo
				(newCollection, odbType, nonNatives);
			coi.SetRealCollectionClassName(realCollectionClassName);
			return coi;
		}
	}
}
