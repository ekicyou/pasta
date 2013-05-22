using System;
namespace NeoDatis.Odb.Impl.Core.Query.List.Objects
{
	/// <summary>
	/// A collection that uses a BTree as an underlying system to provide ordered by Collections
	/// <p>
	/// </p>
	/// </summary>
	[System.Serializable]
	public abstract class AbstractBTreeCollection<E> : NeoDatis.Odb.Objects<E>
	{
		private NeoDatis.Btree.IBTree tree;

		private int size;

		[System.NonSerialized]
		private System.Collections.Generic.IEnumerator<E> currentIterator;

		private NeoDatis.Odb.Core.OrderByConstants orderByType;

		public AbstractBTreeCollection(int size, NeoDatis.Odb.Core.OrderByConstants orderByType
			)
		{
			// TODO compute degree best value for the size value
			tree = BuildTree(NeoDatis.Odb.OdbConfiguration.GetDefaultIndexBTreeDegree());
			this.orderByType = orderByType;
		}

		public AbstractBTreeCollection() : this(50, NeoDatis.Odb.Core.OrderByConstants.OrderByNone
			)
		{
		}

		public abstract NeoDatis.Btree.IBTree BuildTree(int degree);

		public virtual E GetFirst()
		{
			return Iterator(orderByType).Current;
		}

		public virtual bool HasNext()
		{
			if (currentIterator == null)
			{
				currentIterator = Iterator(orderByType);
			}
			return currentIterator.MoveNext();
		}

		public virtual E Next()
		{
			if (currentIterator == null)
			{
				currentIterator = Iterator(orderByType);
			}
			return currentIterator.Current;
		}

		public void Add(E o)
		{
			tree.Insert(size, o);
			size++;
		}

		/// <summary>Adds the object in the btree with the specific key</summary>
		/// <param name="key"></param>
		/// <param name="o"></param>
		/// <returns></returns>
		public virtual bool AddWithKey(NeoDatis.Tool.Wrappers.OdbComparable key, E o)
		{
			tree.Insert(key, o);
			size++;
			return true;
		}

		/// <summary>Adds the object in the btree with the specific key</summary>
		/// <param name="key"></param>
		/// <param name="o"></param>
		/// <returns></returns>
		public virtual bool AddWithKey(int key, E o)
		{
			tree.Insert(key, o);
			size++;
			return true;
		}

		public virtual bool AddAll(System.Collections.Generic.ICollection<E> collection)
		{
			System.Collections.Generic.IEnumerator<E> iterator = collection.GetEnumerator();
			while (iterator.MoveNext())
			{
				Add(iterator.Current);
			}
			return true;
		}

		public virtual void Clear()
		{
			tree.Clear();
		}

		public virtual bool Contains(E o)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				.AddParameter("contains"));
		}

		public virtual bool ContainsAll(System.Collections.ICollection collection)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				.AddParameter("containsAll"));
		}

		public virtual bool IsEmpty()
		{
			return size == 0;
		}

		public virtual System.Collections.Generic.IEnumerator<E> GetEnumerator()
		{
			return Iterator(orderByType);
		}
        System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator()
        {

            return tree.Iterator<object>(orderByType);
        }
		public virtual System.Collections.Generic.IEnumerator<E> Iterator(NeoDatis.Odb.Core.OrderByConstants
			 newOrderByType)
		{
			return (System.Collections.Generic.IEnumerator<E>)tree.Iterator<E>(newOrderByType);
		}

		public virtual bool Remove(E o)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				.AddParameter("remove"));
		}

		public virtual bool RemoveAll(System.Collections.ICollection collection)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				.AddParameter("removeAll"));
		}

		public virtual bool RetainAll(System.Collections.ICollection collection)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				.AddParameter("retainAll"));
		}

		public virtual int Count
		{
			get
			{
				return size;
			}
		}
		
		public virtual bool IsReadOnly
		{
			get
			{
				return true;
			}
		}

		public virtual object[] ToArray()
		{
			return ToArray(new object[size]);
		}

		public virtual object[] ToArray(object[] objects)
		{
			System.Collections.IEnumerator iterator = GetEnumerator();
			int i = 0;
			while (iterator.MoveNext())
			{
				objects[i++] = iterator.Current;
			}
			return objects;
		}

		public virtual void Reset()
		{
			currentIterator = Iterator(orderByType);
		}

		protected virtual NeoDatis.Odb.Core.OrderByConstants GetOrderByType()
		{
			return orderByType;
		}

		protected virtual NeoDatis.Btree.IBTree GetTree()
		{
			return tree;
		}

		public override string ToString()
		{
			System.Text.StringBuilder s = new System.Text.StringBuilder();
			s.Append("size=").Append(size).Append(" [");
			System.Collections.Generic.IEnumerator<E> iterator = GetEnumerator();
			while (iterator.MoveNext())
			{
				s.Append(iterator.Current);
				if (iterator.MoveNext())
				{
					s.Append(" , ");
				}
			}
			s.Append("]");
			return s.ToString();
		}
		
		public void CopyTo(E[] ee,int arrayIndex)
		{
      		  throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				.AddParameter("CopyTo"));
		}
        public void AddOid(OID oid){
            throw new Exception("Add Oid not implemented ");
        }
	}
}
