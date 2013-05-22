namespace NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector
{
	/// <author>olivier</author>
	public class GetDependentObjectIntrospectingCallback : NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
	{
		private NeoDatis.Tool.Wrappers.Map.OdbHashMap<object, object> objects;

		public GetDependentObjectIntrospectingCallback()
		{
			objects = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<object, object>();
		}

		public virtual bool ObjectFound(object o)
		{
			if (o == null)
			{
				return false;
			}
			if (objects.ContainsKey(o))
			{
				return false;
			}
			objects.Add(o, o);
			return true;
		}

		public virtual System.Collections.Generic.ICollection<object> GetObjects()
		{
			return objects.Values;
		}
	}
}
