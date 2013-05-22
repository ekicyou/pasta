namespace NeoDatis.Odb.Impl.Core.Server.Trigger
{
	public class DefaultServerTriggerManager : NeoDatis.Odb.Impl.Core.Trigger.DefaultTriggerManager
	{
		/// <param name="engine"></param>
		public DefaultServerTriggerManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine) : base(engine)
		{
		}

		// TODO Auto-generated constructor stub
		protected override bool IsNull(object @object)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)@object;
			return nnoi == null || nnoi.IsNull();
		}

		public override object Transform(object @object)
		{
			return new NeoDatis.Odb.Impl.Core.Server.Trigger.DefaultObjectRepresentation((NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)@object);
		}
	}
}
