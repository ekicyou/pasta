namespace NeoDatis.Odb.Impl.Core.Server.Trigger
{
	public class DefaultObjectRepresentation : NeoDatis.Odb.ObjectRepresentation
	{
		private readonly NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi;

		public DefaultObjectRepresentation(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			this.nnoi = nnoi;
		}

		public virtual object GetValueOf(string attributeName)
		{
			if (nnoi.IsNull())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.TriggerCalledOnNullObject
					.AddParameter(nnoi.GetClassInfo().GetFullClassName()).AddParameter(attributeName
					));
			}
			return nnoi.GetValueOf(attributeName);
		}

		public virtual void SetValueOf(string attributeName, object value)
		{
			//fixme : storage engine is null?
			NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector introspector = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetLocalObjectIntrospector(null);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = introspector.GetMetaRepresentation
				(value, null, true, null, new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrumentationCallback
				());
			nnoi.SetValueOf(attributeName, aoi);
		}

		public virtual NeoDatis.Odb.OID GetOid()
		{
			return this.nnoi.GetOid();
		}

		public virtual string GetObjectClassName()
		{
			return nnoi.GetClassInfo().GetFullClassName();
		}

		public NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetNnoi()
		{
			return nnoi;
		}
	}
}
