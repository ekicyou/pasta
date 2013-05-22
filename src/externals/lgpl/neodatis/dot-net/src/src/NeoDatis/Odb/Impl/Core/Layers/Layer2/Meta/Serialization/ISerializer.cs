namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization
{
	public interface ISerializer
	{
		string ToString(object @object);

		/// <exception cref="System.Exception"></exception>
		object FromString(string data);
	}
}
