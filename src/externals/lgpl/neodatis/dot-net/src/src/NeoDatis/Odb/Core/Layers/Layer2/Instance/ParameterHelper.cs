namespace NeoDatis.Odb.Core.Layers.Layer2.Instance
{
	/// <summary>Provide constructor parameters for constructors which don't allow nulls as default values.
	/// 	</summary>
	/// <remarks>Provide constructor parameters for constructors which don't allow nulls as default values. Used by the InstanceBuilder.
	/// 	</remarks>
	public interface ParameterHelper
	{
		object[] Parameters();
	}
}
