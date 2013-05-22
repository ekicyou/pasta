namespace NeoDatis.Odb.Core.Lookup
{
	/// <summary>A simple class to enable direct object lookup by object id</summary>
	/// <author>olivier</author>
	public class LookupImpl : NeoDatis.Odb.Core.Lookup.ILookup
	{
		private System.Collections.Generic.IDictionary<string, object> objects;

		public LookupImpl()
		{
			objects = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, object>();
		}

		public virtual object Get(string objectId)
		{
			return objects[objectId];
		}

		public virtual void Set(string objectId, object @object)
		{
			objects.Add(objectId, @object);
		}

		public virtual int Size()
		{
			return objects.Count;
		}
	}
}
