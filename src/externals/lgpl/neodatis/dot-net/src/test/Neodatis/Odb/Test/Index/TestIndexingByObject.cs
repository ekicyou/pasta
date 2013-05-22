using NUnit.Framework;
namespace NeoDatis.Odb.Test.Index
{
	/// <summary>
	/// Junit to test indexing an object when the index field is an object and not a
	/// native attribute
	/// </summary>
	[TestFixture]
    public class TestIndexingByObject : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			Println("************START OF TEST1***************");
			string baseName = GetBaseName();
			DeleteBase("index-object");
			NeoDatis.Odb.ODB odb = Open("index-object");
			string[] fields = new string[] { "object" };
			odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject2)).AddUniqueIndexOn
				("index1", fields, true);
			NeoDatis.Odb.Test.Index.IndexedObject2 o1 = new NeoDatis.Odb.Test.Index.IndexedObject2
				("Object1", new NeoDatis.Odb.Test.Index.IndexedObject("Inner Object 1", 10, new 
				System.DateTime()));
			odb.Store(o1);
			odb.Close();
			odb = Open("index-object");
			// First get the object used to index
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.Index.IndexedObject
				));
			NeoDatis.Odb.Test.Index.IndexedObject io = (NeoDatis.Odb.Test.Index.IndexedObject
				)objects.GetFirst();
			NeoDatis.Odb.Core.Query.IQuery q = odb.CriteriaQuery(typeof(NeoDatis.Odb.Test.Index.IndexedObject2
				), NeoDatis.Odb.Core.Query.Criteria.Where.Equal("object", io));
			objects = odb.GetObjects(q);
			NeoDatis.Odb.Test.Index.IndexedObject2 o2 = (NeoDatis.Odb.Test.Index.IndexedObject2
				)objects.GetFirst();
			odb.Close();
			AssertEquals(o1.GetName(), o2.GetName());
			Println(q.GetExecutionPlan().GetDetails());
			AssertFalse(q.GetExecutionPlan().GetDetails().IndexOf("index1") == -1);
			DeleteBase("index-object");
			Println("************END OF TEST1***************");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			Println("************START OF TEST2***************");
			DeleteBase("index-object");
			NeoDatis.Odb.ODB odb = Open("index-object");
			string[] fields = new string[] { "object" };
			odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject2)).AddUniqueIndexOn
				("index1", fields, true);
			int size = isLocal ? 5000 : 500;
			for (int i = 0; i < size; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.Index.IndexedObject2("Object " + i, new NeoDatis.Odb.Test.Index.IndexedObject
					("Inner Object " + i, i, new System.DateTime())));
			}
			odb.Close();
			odb = Open("index-object");
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "Inner Object " + (size - 1)));
			// First get the object used to index, the last one. There is no index
			// on the class and field
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Objects objects = odb.GetObjects(q);
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Test.Index.IndexedObject io = (NeoDatis.Odb.Test.Index.IndexedObject
				)objects.GetFirst();
			Println("d0=" + (end0 - start0));
			Println(q.GetExecutionPlan().GetDetails());
			q = odb.CriteriaQuery(typeof(NeoDatis.Odb.Test.Index.IndexedObject2), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("object", io));
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			objects = odb.GetObjects(q);
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("d=" + (end - start));
			NeoDatis.Odb.Test.Index.IndexedObject2 o2 = (NeoDatis.Odb.Test.Index.IndexedObject2
				)objects.GetFirst();
			odb.Close();
			AssertEquals("Object " + (size - 1), o2.GetName());
			Println(q.GetExecutionPlan().GetDetails());
			AssertTrue(q.GetExecutionPlan().UseIndex());
			DeleteBase("index-object");
			Println("************END OF TEST2***************");
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3_BadAttributeInIndex()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			string fieldName = "fkjdsfkjdhfjkdhjkdsh";
			try
			{
				odb = Open(baseName);
				string[] fields = new string[] { fieldName };
				odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject2)).AddUniqueIndexOn
					("index1", fields, true);
				Fail("Should have thrown an exception because the field " + fieldName + " does not exist"
					);
			}
			catch (System.Exception)
			{
			}
			finally
			{
				// normal
				odb.Close();
				DeleteBase(baseName);
			}
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test4()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			string[] fields = new string[] { "object" };
			odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject2)).AddUniqueIndexOn
				("index1", fields, true);
			string[] fields2 = new string[] { "name" };
			odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject)).AddUniqueIndexOn
				("index2", fields2, true);
			int size = isLocal ? 5000 : 500;
			for (int i = 0; i < size; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.Index.IndexedObject2("Object " + i, new NeoDatis.Odb.Test.Index.IndexedObject
					("Inner Object " + i, i, new System.DateTime())));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "Inner Object " + (size - 1)));
			// First get the object used to index, the last one. There is no index
			// on the class and field
			long start0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Objects objects = odb.GetObjects(q);
			long end0 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// check if index has been used
			AssertTrue(q.GetExecutionPlan().UseIndex());
			NeoDatis.Odb.Test.Index.IndexedObject io = (NeoDatis.Odb.Test.Index.IndexedObject
				)objects.GetFirst();
			Println("d0=" + (end0 - start0));
			Println(q.GetExecutionPlan().GetDetails());
			q = odb.CriteriaQuery(typeof(NeoDatis.Odb.Test.Index.IndexedObject2), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("object", io));
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			objects = odb.GetObjects(q);
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("d=" + (end - start));
			NeoDatis.Odb.Test.Index.IndexedObject2 o2 = (NeoDatis.Odb.Test.Index.IndexedObject2
				)objects.GetFirst();
			odb.Close();
			AssertEquals("Object " + (size - 1), o2.GetName());
			Println(q.GetExecutionPlan().GetDetails());
			AssertTrue(q.GetExecutionPlan().UseIndex());
			DeleteBase(baseName);
		}
	}
}
