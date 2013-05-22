using NUnit.Framework;
namespace NeoDatis.Odb.Test.Trigger
{
	public class LocalAutoIncrementTrigger : NeoDatis.Odb.Core.Trigger.InsertTrigger
	{
		public override void AfterInsert(object @object, NeoDatis.Odb.OID oid)
		{
		}

		// nothing
		public override bool BeforeInsert(object @object)
		{
			if (@object.GetType() != typeof(NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId
				))
			{
				return false;
			}
			NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId o = (NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId
				)@object;
			NeoDatis.Tool.Mutex.Mutex mutex = NeoDatis.Tool.Mutex.MutexFactory.Get("auto increment mutex"
				);
			try
			{
				try
				{
					mutex.Acquire("trigger");
					long id = GetNextId("test");
					o.SetId(id);
					// System.out.println("setting new id "+ id);
					return true;
				}
				catch (System.Exception e)
				{
					// TODO Auto-generated catch block
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						, e);
				}
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("trigger");
				}
			}
		}

		/// <summary>
		/// Actually gets the next id Gets the object of type ID from the database
		/// with the specific name.
		/// </summary>
		/// <remarks>
		/// Actually gets the next id Gets the object of type ID from the database
		/// with the specific name. Then increment the id value and returns. If
		/// object does not exist, creates t.
		/// </remarks>
		/// <param name="idName"></param>
		/// <returns></returns>
		private long GetNextId(string idName)
		{
			NeoDatis.Odb.ODB odb = GetOdb();
			NeoDatis.Odb.Objects objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Trigger.ID), NeoDatis.Odb.Core.Query.Criteria.Where.Equal
				("idName", idName)));
			if (objects.IsEmpty())
			{
				NeoDatis.Odb.Test.Trigger.ID id1 = new NeoDatis.Odb.Test.Trigger.ID(idName, 1);
				odb.Store(id1);
				return 1;
			}
			NeoDatis.Odb.Test.Trigger.ID id = (NeoDatis.Odb.Test.Trigger.ID)objects.GetFirst(
				);
			long lid = id.GetNext();
			odb.Store(id);
			return lid;
		}
	}
}
