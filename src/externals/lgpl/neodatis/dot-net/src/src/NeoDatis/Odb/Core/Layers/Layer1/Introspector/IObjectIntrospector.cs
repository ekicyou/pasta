namespace NeoDatis.Odb.Core.Layers.Layer1.Introspector
{
	/// <summary>Interface for ObjectInstropector.</summary>
	/// <remarks>Interface for ObjectInstropector. It has local and Client/Server implementation.
	/// 	</remarks>
	/// <author>osmadja</author>
	public interface IObjectIntrospector
	{
		/// <summary>retrieve object data</summary>
		/// <param name="@object">The object to get meta representation</param>
		/// <param name="ci">The ClassInfo</param>
		/// <param name="recursive">To indicate that introspection must be recursive</param>
		/// <param name="alreadyReadObjects">A map with already read object, to avoid cyclic reference problem
		/// 	</param>
		/// <returns>The object info</returns>
		NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo GetMetaRepresentation(object
			 @object, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci, bool recursive, System.Collections.Generic.IDictionary
			<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo> alreadyReadObjects
			, NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback callback);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo BuildNnoi(object @object
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			[] values, long[] attributesIdentification, int[] attributeIds, System.Collections.Generic.IDictionary
			<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo> alreadyReadObjects
			);

		void Clear();
	}
}
