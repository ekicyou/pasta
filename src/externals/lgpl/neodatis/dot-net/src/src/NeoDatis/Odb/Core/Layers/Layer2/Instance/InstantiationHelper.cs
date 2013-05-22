namespace NeoDatis.Odb.Core.Layers.Layer2.Instance
{
	/// <summary>Create an instance of a class in some way which can't be managed just via reflection
	/// 	</summary>
	public interface InstantiationHelper
	{
		object Instantiate();
	}
}
