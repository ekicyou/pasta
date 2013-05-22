namespace NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector
{
	/// <summary>The ClassIntrospector is used to introspect classes.</summary>
	/// <remarks>
	/// The ClassIntrospector is used to introspect classes. It uses Reflection to
	/// extract class information. It transforms a native Class into a ClassInfo (a
	/// meta representation of the class) that contains all informations about the
	/// class.
	/// </remarks>
	/// <author>osmadja</author>
	public class AndroidClassIntrospector : NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.AbstractClassIntrospector
	{
		public AndroidClassIntrospector() : base()
		{
		}

		/// <summary>
		/// NeoDatis uses sun classes to create dynamic empty constructors so it does not work on Android
		/// TODO check how to do this on Android
		/// and stores it the constructor cache.
		/// </summary>
		/// <remarks>
		/// NeoDatis uses sun classes to create dynamic empty constructors so it does not work on Android
		/// TODO check how to do this on Android
		/// and stores it the constructor cache.
		/// </remarks>
		/// <param name="clazz"></param>
		/// <returns></returns>
		protected override bool TryToCreateAnEmptyConstructor(System.Type clazz)
		{
			return false;
		}
	}
}
