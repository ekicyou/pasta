namespace NeoDatis.Odb.Core.Lookup
{
	/// <author>olivier</author>
	public interface ILookup
	{
		object Get(string objectId);

		void Set(string objectId, object @object);

		int Size();
	}
}
