using System;
namespace NeoDatis.Odb.Impl.Core.Query.List.Objects
{
	/// <summary>A simple list to hold query result.</summary>
	/// <remarks>A simple list to hold query result. It is used when no index and no order by is used and inMemory = true
	/// 	</remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class SimpleList<E> : System.Collections.Generic.List<E>, NeoDatis.Odb.Objects<E>
	{
		private int currentPosition;

		public SimpleList() : base()
		{
		}

		public SimpleList(int initialCapacity) : base(initialCapacity)
		{
		}

		public virtual bool AddWithKey(NeoDatis.Tool.Wrappers.OdbComparable key, E o)
		{
			Add(o);
			return true;
		}

		public virtual bool AddWithKey(int key, E o)
		{
			Add(o);
			return true;
		}

		public virtual E GetFirst()
		{
			return this[0];
		}

		public virtual bool HasNext()
		{
			return currentPosition < Count;
		}

		/// <summary>The orderByType in not supported by this kind of list</summary>
		public virtual System.Collections.Generic.IEnumerator<E> Iterator(NeoDatis.Odb.Core.OrderByConstants
			 orderByType)
		{
			return GetEnumerator();
		}

		public virtual E Next()
		{
			return this[currentPosition++];
		}

		public virtual void Reset()
		{
			currentPosition = 0;
		}
        public void AddOid(OID oid)
        {
            throw new Exception("Add Oid not implemented ");
        }
	}
}
