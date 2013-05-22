using NeoDatis.Tool.Wrappers.List;
using System.Collections.Generic;
namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>The database meta-model</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public abstract class MetaModel
	{
		/// <summary>A hash map to speed up the access of classinfo by full class name</summary>
		private System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> rapidAccessForUserClassesByName;

		private System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> rapidAccessForSystemClassesByName;

		private System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> rapidAccessForClassesByOid;

		/// <summary>A simple list to hold all class infos.</summary>
		/// <remarks>A simple list to hold all class infos. It is redundant with the maps, but in some cases, we need sequential access to classes :-(
		/// 	</remarks>
		private NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> allClassInfos;

		/// <summary>to identify if meta model has changed</summary>
		private bool hasChanged;

		[System.NonSerialized]
		protected NeoDatis.Odb.Core.Layers.Layer2.Instance.IClassPool classPool;

		public MetaModel()
		{
			this.classPool = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassPool();
			rapidAccessForUserClassesByName = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string
				, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo>(10);
			rapidAccessForSystemClassesByName = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string
				, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo>(10);
			rapidAccessForClassesByOid = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Odb.OID
				, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo>(10);
			allClassInfos = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
				>();
		}

		public virtual void AddClass(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo
			)
		{
			if (classInfo.IsSystemClass())
			{
				rapidAccessForSystemClassesByName.Add(classInfo.GetFullClassName(), classInfo);
			}
			else
			{
				rapidAccessForUserClassesByName.Add(classInfo.GetFullClassName(), classInfo);
			}
			rapidAccessForClassesByOid.Add(classInfo.GetId(), classInfo);
			allClassInfos.Add(classInfo);
		}

		public virtual void AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 ciList)
		{
			System.Collections.IEnumerator iterator = ciList.GetClassInfos().GetEnumerator();
			while (iterator.MoveNext())
			{
				AddClass((NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator.Current);
			}
		}

		public virtual bool ExistClass(string fullClassName)
		{
			// Check if it is a system class
			bool exist = rapidAccessForSystemClassesByName.ContainsKey(fullClassName);
			if (exist)
			{
				return true;
			}
			// Check if it is user class
			exist = rapidAccessForUserClassesByName.ContainsKey(fullClassName);
			return exist;
		}

		public override string ToString()
		{
			return rapidAccessForUserClassesByName.Values + "/" + rapidAccessForSystemClassesByName
				.Values;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> GetAllClasses()
		{
			return allClassInfos;
		}

		public virtual System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> GetUserClasses()
		{
			return rapidAccessForUserClassesByName.Values;
		}

		public virtual System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> GetSystemClasses()
		{
			return rapidAccessForSystemClassesByName.Values;
		}

		public virtual int GetNumberOfClasses()
		{
			return allClassInfos.Count;
		}

		public virtual int GetNumberOfUserClasses()
		{
			return rapidAccessForUserClassesByName.Count;
		}

		public virtual int GetNumberOfSystemClasses()
		{
			return rapidAccessForSystemClassesByName.Count;
		}

		/// <summary>Gets the class info from the OID.</summary>
		/// <remarks>Gets the class info from the OID.</remarks>
		/// <param name="id"></param>
		/// <returns>the class info with the OID</returns>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo GetClassInfoFromId(
			NeoDatis.Odb.OID id)
		{
			return rapidAccessForClassesByOid[id];
		}

		public virtual ClassInfo GetClassInfo(string fullClassName, bool throwExceptionIfDoesNotExist)
		{
			// Check if it is a system class
            NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = null;

            rapidAccessForSystemClassesByName.TryGetValue(fullClassName, out ci);
			
            if (ci != null)
			{
				return ci;
			}
			// Check if it is user class
			rapidAccessForUserClassesByName.TryGetValue(fullClassName,out ci);
			if (ci != null)
			{
				return ci;
			}
			if (throwExceptionIfDoesNotExist)
			{
				throw new ODBRuntimeException(NeoDatisError.MetaModelClassNameDoesNotExist.AddParameter(fullClassName));
			}
			return null;
		}

		/// <returns>The Last class info</returns>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo GetLastClassInfo()
		{
			return allClassInfos[allClassInfos.Count - 1];
		}

		/// <summary>This method is only used by the odb explorer.</summary>
		/// <remarks>
		/// This method is only used by the odb explorer. So there is no too much
		/// problem with performance issue.
		/// </remarks>
		/// <param name="ci"></param>
		/// <returns>The index of the class info</returns>
		public virtual int SlowGetUserClassInfoIndex(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 ci)
		{
			System.Collections.IEnumerator iterator = rapidAccessForUserClassesByName.Values.
				GetEnumerator();
			int i = 0;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci2 = null;
			while (iterator.MoveNext())
			{
				ci2 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator.Current;
				if (ci2.GetId() == ci.GetId())
				{
					return i;
				}
				i++;
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClassInfoDoesNotExistInMetaModel
				.AddParameter(ci.GetFullClassName()));
		}

		/// <param name="index">The index of the class info to get</param>
		/// <returns>The class info at the specified index</returns>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo GetClassInfo(int index
			)
		{
			return allClassInfos[index];
		}

		/// <summary>The method is slow nut it is only used in the odb explorer.</summary>
		/// <remarks>The method is slow nut it is only used in the odb explorer.</remarks>
		/// <param name="index"></param>
		/// <returns></returns>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo SlowGetUserClassInfo
			(int index)
		{
			System.Collections.IEnumerator iterator = rapidAccessForUserClassesByName.Values.
				GetEnumerator();
			int i = 0;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = null;
			while (iterator.MoveNext())
			{
				ci = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator.Current;
				if (i == index)
				{
					return ci;
				}
				i++;
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClassInfoDoesNotExistInMetaModel
				.AddParameter(" with index " + index));
		}

		public virtual void Clear()
		{
			rapidAccessForSystemClassesByName.Clear();
			rapidAccessForUserClassesByName.Clear();
			rapidAccessForSystemClassesByName = null;
			rapidAccessForUserClassesByName = null;
			allClassInfos.Clear();
		}

		public virtual bool HasChanged()
		{
			return hasChanged;
		}

		public virtual void SetHasChanged(bool hasChanged)
		{
			this.hasChanged = hasChanged;
		}

		public abstract System.Collections.Generic.ICollection<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> GetChangedClassInfo();

		public abstract void ResetChangedClasses();

		/// <summary>
		/// Saves the fact that something has changed in the class (number of
		/// objects or last object oid)
		/// </summary>
		/// <param name="ci"></param>
		public abstract void AddChangedClass(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 ci);

		public virtual System.Collections.Generic.IDictionary<string, object> GetHistory(
			)
		{
			System.Collections.Generic.IDictionary<string, object> map = new NeoDatis.Tool.Wrappers.Map.OdbHashMap
				<string, object>();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = null;
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
				> iterator = allClassInfos.GetEnumerator();
			while (iterator.MoveNext())
			{
				ci = iterator.Current;
				map.Add(ci.GetFullClassName(), ci.GetHistory());
			}
			return map;
		}

		/// <summary>Builds a meta model from a list of class infos</summary>
		/// <param name="classInfos"></param>
		/// <returns>The new Metamodel</returns>
		public static NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel FromClassInfos(NeoDatis.Tool.Wrappers.List.IOdbList
			<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo> classInfos)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = new NeoDatis.Odb.Core.Layers.Layer2.Meta.SessionMetaModel
				();
			int nbClasses = classInfos.Count;
			for (int i = 0; i < nbClasses; i++)
			{
				metaModel.AddClass(classInfos[i]);
			}
			return metaModel;
		}

		/// <summary>Gets all the persistent classes that are subclasses or equal to the parameter class
		/// 	</summary>
		/// <param name="fullClassName"></param>
		/// <returns>The list of class info of persistent classes that are subclasses or equal to the class
		/// 	</returns>
		public virtual IOdbList<ClassInfo > GetPersistentSubclassesOf(string fullClassName)
		{
			IOdbList<ClassInfo> result = new OdbArrayList<ClassInfo>();
			IEnumerator<string> classNames = rapidAccessForUserClassesByName.Keys.GetEnumerator();
			string oneClassName = null;
			System.Type theClass = classPool.GetClass(fullClassName);
			System.Type oneClass = null;
			while (classNames.MoveNext())
			{
				oneClassName = classNames.Current;
				if (oneClassName.Equals(fullClassName))
				{
					result.Add(GetClassInfo(oneClassName, true));
				}
				else
				{
					oneClass = classPool.GetClass(oneClassName);
					if (theClass.IsAssignableFrom(oneClass))
					{
						result.Add(GetClassInfo(oneClassName, true));
					}
				}
			}
			return result;
		}

		/// <returns></returns>
		public abstract NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel Duplicate();
	}
}
