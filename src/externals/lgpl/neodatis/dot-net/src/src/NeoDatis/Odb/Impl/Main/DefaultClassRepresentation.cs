namespace NeoDatis.Odb.Impl.Main
{
	[System.Serializable]
	public class DefaultClassRepresentation : NeoDatis.Odb.ClassRepresentation
	{
		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo;

		private NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector classIntrospector;

		public DefaultClassRepresentation(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine 
			storageEngine, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo) : base(
			)
		{
			this.storageEngine = storageEngine;
			this.classInfo = classInfo;
			this.classIntrospector = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassIntrospector
				();
		}

		public virtual void AddUniqueIndexOn(string name, string[] indexFields, bool verbose
			)
		{
			storageEngine.AddIndexOn(classInfo.GetFullClassName(), name, indexFields, verbose
				, false);
		}

		public virtual void AddIndexOn(string name, string[] indexFields, bool verbose)
		{
			storageEngine.AddIndexOn(classInfo.GetFullClassName(), name, indexFields, verbose
				, true);
		}

		public virtual void AddInstantiationHelper(NeoDatis.Odb.Core.Layers.Layer2.Instance.InstantiationHelper
			 instantiationHelper)
		{
			classIntrospector.AddInstantiationHelper(classInfo.GetFullClassName(), instantiationHelper
				);
		}

		public virtual void AddFullInstantiationHelper(NeoDatis.Odb.Core.Layers.Layer2.Instance.FullInstantiationHelper
			 instantiationHelper)
		{
			classIntrospector.AddFullInstantiationHelper(classInfo.GetFullClassName(), instantiationHelper
				);
		}

		public virtual void AddParameterHelper(NeoDatis.Odb.Core.Layers.Layer2.Instance.ParameterHelper
			 parameterHelper)
		{
			classIntrospector.AddParameterHelper(classInfo.GetFullClassName(), parameterHelper
				);
		}

		public virtual void RemoveInstantiationHelper()
		{
			classIntrospector.RemoveInstantiationHelper(classInfo.GetFullClassName());
		}

		public virtual void RemoveFullInstantiationHelper()
		{
			classIntrospector.RemoveInstantiationHelper(classInfo.GetFullClassName());
		}

		public virtual void RemoveParameterHelper()
		{
			classIntrospector.RemoveParameterHelper(classInfo.GetFullClassName());
		}

		public virtual bool ExistIndex(string indexName)
		{
			return classInfo.HasIndex(indexName);
		}

		/// <summary>Used to rebuild an index</summary>
		public virtual void RebuildIndex(string indexName, bool verbose)
		{
			storageEngine.RebuildIndex(classInfo.GetFullClassName(), indexName, verbose);
		}

		public virtual void DeleteIndex(string indexName, bool verbose)
		{
			storageEngine.DeleteIndex(classInfo.GetFullClassName(), indexName, verbose);
		}
	}
}
