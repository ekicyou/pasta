namespace NeoDatis.Odb.Core.Layers.Layer1.Introspector
{
	public interface IClassIntrospector : NeoDatis.Odb.Core.ITwoPhaseInit
	{
		void Reset();

		void AddInstanciationHelper(System.Type clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.InstantiationHelper
			 helper);

		void AddParameterHelper(System.Type clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.ParameterHelper
			 helper);

		void AddFullInstanciationHelper(System.Type clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.FullInstantiationHelper
			 helper);

		void AddInstantiationHelper(string clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.InstantiationHelper
			 helper);

		void AddParameterHelper(string clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.ParameterHelper
			 helper);

		void AddFullInstantiationHelper(string clazz, NeoDatis.Odb.Core.Layers.Layer2.Instance.FullInstantiationHelper
			 helper);

		void RemoveInstantiationHelper(System.Type clazz);

		void RemoveInstantiationHelper(string canonicalName);

		void RemoveParameterHelper(System.Type clazz);

		void RemoveParameterHelper(string canonicalName);

		void RemoveFullInstantiationHelper(System.Type clazz);

		void RemoveFullInstantiationHelper(string canonicalName);

		/// <summary>introspect a list of classes</summary>
		/// <param name="classInfos"></param>
		/// <returns>A map where the key is the class name and the key is the ClassInfo: the class meta representation
		/// 	</returns>
		System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> Instrospect(NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> classInfos);

		/// <param name="clazz">The class to instrospect</param>
		/// <param name="recursive">If true, goes does the hierarchy to try to analyse all classes
		/// 	</param>
		/// <returns>The list of class info detected while introspecting the class</returns>
		NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList Introspect(System.Type clazz, 
			bool recursive);

		/// <summary>
		/// Builds a class info from a class and an existing class info
		/// <pre>
		/// The existing class info is used to make sure that fields with the same name will have
		/// the same id
		/// </pre>
		/// </summary>
		/// <param name="fullClassName">The name of the class to get info</param>
		/// <param name="existingClassInfo"></param>
		/// <returns>A ClassInfo - a meta representation of the class</returns>
		NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo GetClassInfo(string fullClassName, 
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo existingClassInfo);

		/// <param name="fullClassName"></param>
		/// <param name="includingThis"></param>
		/// <returns>The list of super classes</returns>
		System.Collections.IList GetSuperClasses(string fullClassName, bool includingThis
			);

		NeoDatis.Tool.Wrappers.List.IOdbList<System.Reflection.FieldInfo> GetAllFields(string
			 fullClassName);

		NeoDatis.Tool.Wrappers.List.IOdbList<System.Reflection.FieldInfo> RemoveUnnecessaryFields
			(NeoDatis.Tool.Wrappers.List.IOdbList<System.Reflection.FieldInfo> fields);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList Introspect(string fullClassName
			, bool recursive);

		System.Reflection.ConstructorInfo GetConstructorOf(string fullClassName);

		object NewFullInstanceOf(System.Type clazz, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi);

		object NewInstanceOf(System.Type clazz);

		byte GetClassCategory(string fullClassName);

		bool IsSystemClass(string fullClassName);

		System.Reflection.FieldInfo GetField(System.Type clazz, string fieldName);
	}
}
