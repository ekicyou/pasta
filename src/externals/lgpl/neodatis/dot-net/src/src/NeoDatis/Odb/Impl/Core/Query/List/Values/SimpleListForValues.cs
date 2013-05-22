using System;
namespace NeoDatis.Odb.Impl.Core.Query.List.Values
{
	/// <summary>A simple list to hold query result for Object Values API.</summary>
	/// <remarks>A simple list to hold query result for Object Values API. It is used when no index and no order by is used and inMemory = true
	/// 	</remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class SimpleListForValues : NeoDatis.Odb.Impl.Core.Query.List.Objects.SimpleList
		<NeoDatis.Odb.ObjectValues>, NeoDatis.Odb.Values
	{
		public SimpleListForValues() : base()
		{
		}

		public SimpleListForValues(int initialCapacity) : base(initialCapacity)
		{
		}

		public virtual NeoDatis.Odb.ObjectValues NextValues()
		{
			return Next();
		}

		public override bool AddWithKey(NeoDatis.Tool.Wrappers.OdbComparable key, NeoDatis.Odb.ObjectValues
			 @object)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				.AddParameter("addWithKey"));
		}

		public override bool AddWithKey(int key, NeoDatis.Odb.ObjectValues @object)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				.AddParameter("addWithKey"));
		}
        public void AddOid(OID oid)
        {
            throw new Exception("Add Oid not implemented ");
        }
	}
}
