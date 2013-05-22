namespace NeoDatis.Odb.Core.Layers.Layer1.Introspector
{
	/// <author>
	/// olivier
	/// A simple callback used by the introspection API to inform when object are found
	/// </author>
	public interface IIntrospectionCallback
	{
		/// <summary>Called when the introspector find a non native object.</summary>
		/// <remarks>Called when the introspector find a non native object.</remarks>
		/// <param name="@object"></param>
		/// <returns>true to continue going recursively, false do not go deeper</returns>
		bool ObjectFound(object @object);
	}
}
