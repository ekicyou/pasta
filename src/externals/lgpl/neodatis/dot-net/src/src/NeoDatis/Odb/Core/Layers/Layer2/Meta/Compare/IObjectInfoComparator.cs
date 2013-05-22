namespace NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare
{
	public interface IObjectInfoComparator
	{
		bool HasChanged(NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi1, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 aoi2);

		void Clear();

		int GetNbChanges();

		bool SupportInPlaceUpdate();

		System.Collections.Generic.IList<NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedObjectInfo
			> GetChanges();

		System.Collections.Generic.IList<NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.NewNonNativeObjectAction
			> GetNewObjectMetaRepresentations();

		NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.NewNonNativeObjectAction GetNewObjectMetaRepresentation
			(int i);

		System.Collections.Generic.IList<object> GetNewObjects();

		int GetMaxObjectRecursionLevel();

		System.Collections.Generic.IList<NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ChangedAttribute
			> GetChangedAttributeActions();

		System.Collections.Generic.IList<NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.ArrayModifyElement
			> GetArrayChanges();

		System.Collections.Generic.IList<NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare.SetAttributeToNullAction
			> GetAttributeToSetToNull();

		NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo GetChangedObjectMetaRepresentation
			(int i);
	}
}
