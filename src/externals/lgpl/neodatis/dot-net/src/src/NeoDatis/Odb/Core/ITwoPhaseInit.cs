namespace NeoDatis.Odb.Core
{
	/// <summary>A simple interface to guarantee a second init phase for objects.</summary>
	/// <remarks>
	/// A simple interface to guarantee a second init phase for objects. This used the CoreProvider objects
	/// as they can have cyclic reference that can cause cyclic initialization problem. Cyclic initialization
	/// work should be executed in the second init phase to guarantee we are working on
	/// complete initialized instance instead of partially initialized
	/// </remarks>
	/// <author>olivier</author>
	public interface ITwoPhaseInit
	{
		/// <summary>The second init phase</summary>
		void Init2();
	}
}
