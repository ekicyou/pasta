using NeoDatis.Tool.Wrappers.List;
namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>The main implementation of the MetaModel abstract class.</summary>
	/// <remarks>The main implementation of the MetaModel abstract class.</remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class SessionMetaModel : NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel
	{
		/// <summary>
		/// A list of changed classes - that must be persisted back when commit is
		/// done
		/// </summary>
		private NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo> changedClasses;

		public SessionMetaModel() : base()
		{
			changedClasses = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
				, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo>();
		}

		/// <summary>
		/// Saves the fact that something has changed in the class (number of objects
		/// or last object oid)
		/// </summary>
		/// <param name="classInfo"></param>
		/// <param name="uci"></param>
		public override void AddChangedClass(ClassInfo classInfo)
		{
            changedClasses[classInfo] = classInfo;
			SetHasChanged(true);
		}

		public override System.Collections.Generic.ICollection<ClassInfo> GetChangedClassInfo()
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<ClassInfo> l = new OdbArrayList<ClassInfo>();  
			l.AddAll(changedClasses.Keys);
			// TODO return an unmodifianle collection
			// return Collections.unmodifiableCollection(l);
			return l;
		}

		public override void ResetChangedClasses()
		{
			this.changedClasses.Clear();
			SetHasChanged(false);
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel Duplicate()
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.SessionMetaModel model = new NeoDatis.Odb.Core.Layers.Layer2.Meta.SessionMetaModel
				();
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
				> classes = GetAllClasses();
			foreach (NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci in classes)
			{
				model.AddClass((NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)ci.Duplicate(false
					));
			}
			model.changedClasses = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
				, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo>();
			model.changedClasses.PutAll(changedClasses);
			return model;
		}
	}
}
