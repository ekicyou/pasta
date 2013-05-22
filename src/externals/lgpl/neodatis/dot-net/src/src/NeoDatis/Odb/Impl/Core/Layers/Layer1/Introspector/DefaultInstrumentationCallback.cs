namespace NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector
{
	/// <author>olivier</author>
	public class DefaultInstrumentationCallback : NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
	{
		public DefaultInstrumentationCallback() : base()
		{
		}

		public virtual bool ObjectFound(object @object)
		{
			return true;
		}
	}
}
