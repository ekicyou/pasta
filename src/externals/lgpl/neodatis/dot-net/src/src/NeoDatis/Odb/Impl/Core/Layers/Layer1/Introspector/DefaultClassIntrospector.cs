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
	public class DefaultClassIntrospector : NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.AbstractClassIntrospector
	{
		public DefaultClassIntrospector() : base()
		{
		}

		/// <summary>
		/// Tries to create a default constructor (with no parameter) for the class
		/// and stores it the constructor cache.
		/// </summary>
		/// <remarks>
		/// Tries to create a default constructor (with no parameter) for the class
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
