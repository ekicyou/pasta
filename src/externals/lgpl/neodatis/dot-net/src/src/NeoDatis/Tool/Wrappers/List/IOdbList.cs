namespace NeoDatis.Tool.Wrappers.List
{
	/// <author>olivier</author>
	public interface IOdbList<E> : System.Collections.Generic.IList<E>
	{
		bool AddAll(System.Collections.Generic.ICollection<E> c);

		bool RemoveAll(System.Collections.Generic.ICollection<E> c);

		void Add(E o);

		E Get(int index);
		void Set(int index, E element);

		bool IsEmpty();
	}
}
