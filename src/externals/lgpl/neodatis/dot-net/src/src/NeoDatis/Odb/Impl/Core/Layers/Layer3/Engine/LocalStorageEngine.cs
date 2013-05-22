namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	public class LocalStorageEngine : NeoDatis.Odb.Core.Layers.Layer3.Engine.AbstractStorageEngine
	{
		protected NeoDatis.Odb.Core.Transaction.ISession session;

		public LocalStorageEngine(NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification parameters
			) : base(parameters)
		{
		}

		public override NeoDatis.Odb.Core.Transaction.ISession BuildDefaultSession()
		{
			session = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetLocalSession(this);
			return session;
		}

		public override NeoDatis.Odb.Core.Transaction.ISession GetSession(bool throwExceptionIfDoesNotExist
			)
		{
			return session;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 classInfoList)
		{
			return GetObjectWriter().AddClasses(classInfoList);
		}

		public override NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			BuildObjectIntrospector()
		{
			return provider.GetLocalObjectIntrospector(this);
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter BuildObjectWriter()
		{
			return provider.GetClientObjectWriter(this);
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IObjectReader BuildObjectReader()
		{
			return provider.GetClientObjectReader(this);
		}

		public override NeoDatis.Odb.Core.Trigger.ITriggerManager BuildTriggerManager()
		{
			return provider.GetLocalTriggerManager(this);
		}
	}
}
