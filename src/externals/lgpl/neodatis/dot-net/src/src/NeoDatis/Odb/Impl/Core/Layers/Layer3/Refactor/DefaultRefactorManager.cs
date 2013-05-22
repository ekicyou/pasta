namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Refactor
{
	public class DefaultRefactorManager : NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager
	{
		protected NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		public DefaultRefactorManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine
			)
		{
			this.storageEngine = storageEngine;
		}

		public virtual void AddField(string className, System.Type fieldType, string fieldName
			)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = storageEngine.GetSession
				(true).GetMetaModel();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = metaModel.GetClassInfo(className
				, true);
			// The real attribute id (-1) will be set in the ci.addAttribute
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai = new NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo
				(-1, fieldName, fieldType.FullName, ci);
			ci.AddAttribute(cai);
			storageEngine.GetObjectWriter().UpdateClassInfo(ci, true);
		}

		public virtual void ChangeFieldType(string className, string attributeName, System.Type
			 newType)
		{
		}

		// TODO Auto-generated method stub
		public virtual void RemoveClass(string className)
		{
		}

		// TODO Auto-generated method stub
		/// <exception cref="System.IO.IOException"></exception>
		public virtual void RemoveField(string className, string attributeName)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = storageEngine.GetSession
				(true).GetMetaModel();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = metaModel.GetClassInfo(className
				, true);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai2 = ci.GetAttributeInfoFromName
				(attributeName);
			ci.RemoveAttribute(cai2);
			storageEngine.GetObjectWriter().UpdateClassInfo(ci, true);
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void RenameClass(string fullClassName, string newFullClassName)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = storageEngine.GetSession
				(true).GetMetaModel();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = metaModel.GetClassInfo(fullClassName
				, true);
			ci.SetFullClassName(newFullClassName);
			storageEngine.GetObjectWriter().UpdateClassInfo(ci, true);
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void RenameField(string className, string attributeName, string newAttributeName
			)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = storageEngine.GetSession
				(true).GetMetaModel();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = metaModel.GetClassInfo(className
				, true);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassAttributeInfo cai2 = ci.GetAttributeInfoFromName
				(attributeName);
			cai2.SetName(newAttributeName);
			storageEngine.GetObjectWriter().UpdateClassInfo(ci, true);
		}
	}
}
