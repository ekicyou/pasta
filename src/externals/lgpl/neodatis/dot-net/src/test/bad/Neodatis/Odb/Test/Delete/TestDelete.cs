namespace NeoDatis.Odb.Test.Delete
{
	public class TestDelete : NeoDatis.Odb.Test.ODBTest
	{
		public static long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();

		public static string FileName1 = "test-delete.neodatis";

		public static string FileName2 = "test-delete-defrag.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			long n = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Login.Function)));
			NeoDatis.Odb.Test.VO.Login.Function function1 = new NeoDatis.Odb.Test.VO.Login.Function
				("function1");
			NeoDatis.Odb.Test.VO.Login.Function function2 = new NeoDatis.Odb.Test.VO.Login.Function
				("function2");
			NeoDatis.Odb.Test.VO.Login.Function function3 = new NeoDatis.Odb.Test.VO.Login.Function
				("function3");
			odb.Store(function1);
			odb.Store(function2);
			odb.Store(function3);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects l = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "function2")));
			NeoDatis.Odb.Test.VO.Login.Function function = (NeoDatis.Odb.Test.VO.Login.Function
				)l.GetFirst();
			odb.Delete(function);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects l2 = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			AssertEquals(n + 2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			long nbFunctions = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function)));
			long nbProfiles = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Profile)));
			NeoDatis.Odb.Test.VO.Login.Function function1 = new NeoDatis.Odb.Test.VO.Login.Function
				("function1");
			NeoDatis.Odb.Test.VO.Login.Function function2 = new NeoDatis.Odb.Test.VO.Login.Function
				("function2");
			NeoDatis.Odb.Test.VO.Login.Function function3 = new NeoDatis.Odb.Test.VO.Login.Function
				("function3");
			System.Collections.IList functions = new System.Collections.ArrayList();
			functions.Add(function1);
			functions.Add(function2);
			functions.Add(function3);
			NeoDatis.Odb.Test.VO.Login.Profile profile1 = new NeoDatis.Odb.Test.VO.Login.Profile
				("profile1", functions);
			NeoDatis.Odb.Test.VO.Login.Profile profile2 = new NeoDatis.Odb.Test.VO.Login.Profile
				("profile2", function1);
			odb.Store(profile1);
			odb.Store(profile2);
			odb.Close();
			odb = Open(baseName);
			// checks functions
			NeoDatis.Odb.Objects lfunctions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			AssertEquals(nbFunctions + 3, lfunctions.Count);
			NeoDatis.Odb.Objects l = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "function2")));
			NeoDatis.Odb.Test.VO.Login.Function function = (NeoDatis.Odb.Test.VO.Login.Function
				)l.GetFirst();
			odb.Delete(function);
			odb.Close();
			odb = Open(baseName);
			AssertEquals(nbFunctions + 2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			NeoDatis.Odb.Objects l2 = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			// check Profile 1
			NeoDatis.Odb.Objects lprofile = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Profile), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "profile1")));
			NeoDatis.Odb.Test.VO.Login.Profile p1 = (NeoDatis.Odb.Test.VO.Login.Profile)lprofile
				.GetFirst();
			AssertEquals(2, p1.GetFunctions().Count);
			odb.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test30()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.OID oid1 = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function 1"
				));
			NeoDatis.Odb.OID oid2 = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function 2"
				));
			odb.Close();
			Println(oid1);
			Println(oid2);
			odb = Open(baseName);
			odb.Delete(odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).GetFirst()
				);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
				GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).GetFirst();
			odb.Close();
			DeleteBase(baseName);
			AssertEquals("function 2", f.GetName());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test3()
		{
			string baseName = GetBaseName();
			string baseName2 = "2" + baseName;
			NeoDatis.Odb.ODB odb = Open(baseName);
			int size = 1000;
			for (int i = 0; i < size; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function " + i));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), false);
			int j = 0;
			while (objects.HasNext() && j < objects.Count - 1)
			{
				odb.Delete(objects.Next());
				j++;
			}
			odb.Close();
			odb = Open(baseName);
			AssertEquals(1, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			if (isLocal)
			{
				odb = Open(baseName);
				odb.DefragmentTo(NeoDatis.Odb.Test.ODBTest.Directory + baseName2);
				odb.Close();
				odb = Open(baseName2);
				AssertEquals(1, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
				odb.Close();
			}
			DeleteBase(baseName);
			DeleteBase(baseName2);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test4()
		{
			string baseName = GetBaseName();
			int n = isLocal ? 1000 : 10;
			NeoDatis.Odb.ODB odb = Open(baseName);
			long size = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Login.Function)));
			for (int i = 0; i < n; i++)
			{
				NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
					("login - " + (i + 1));
				odb.Store(login);
				AssertEquals(size + i + 1, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			}
			// IStorageEngine engine = Dummy.getEngine(odb);
			odb.Commit();
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			int j = 0;
			while (l.HasNext())
			{
				// println("i="+i);
				NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)l.Next
					();
				odb.Delete(f);
				NeoDatis.Odb.Objects l2 = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
					));
				AssertEquals(size + n - (j + 1), l2.Count);
				j++;
			}
			odb.Commit();
			odb.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test5()
		{
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function("function1"
				);
			odb.Store(f);
			NeoDatis.Odb.OID id = odb.GetObjectId(f);
			try
			{
				odb.Delete(f);
				NeoDatis.Odb.OID id2 = odb.GetObjectId(f);
				Fail("The object has been deleted, the id should have been marked as deleted");
			}
			catch (NeoDatis.Odb.ODBRuntimeException)
			{
				odb.Close();
				DeleteBase(baseName);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test5_byOid()
		{
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function("function1"
				);
			odb.Store(f);
			NeoDatis.Odb.OID oid = odb.GetObjectId(f);
			try
			{
				odb.DeleteObjectWithId(oid);
				NeoDatis.Odb.OID id2 = odb.GetObjectId(f);
				Fail("The object has been deleted, the id should have been marked as deleted");
			}
			catch (NeoDatis.Odb.ODBRuntimeException)
			{
				odb.Close();
				DeleteBase(baseName);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test5_deleteNullObject()
		{
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function("function1"
				);
			odb.Store(f);
			NeoDatis.Odb.OID oid = odb.GetObjectId(f);
			try
			{
				odb.Delete(null);
				Fail("Should have thrown an exception: trying to delete a null object");
			}
			catch (NeoDatis.Odb.ODBRuntimeException)
			{
				odb.Close();
				DeleteBase(baseName);
			}
			catch (System.Exception)
			{
				Fail("Should have thrown an OdbRuntimeException: trying to delete a null object");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test6()
		{
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f = new NeoDatis.Odb.Test.VO.Login.Function("function1"
				);
			odb.Store(f);
			NeoDatis.Odb.OID id = odb.GetObjectId(f);
			odb.Commit();
			try
			{
				odb.Delete(f);
				odb.GetObjectFromId(id);
				Fail("The object has been deleted, the id should have been marked as deleted");
			}
			catch (NeoDatis.Odb.ODBRuntimeException)
			{
				odb.Close();
				DeleteBase("t-delete1.neodatis");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test7()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.Test.VO.Login.Function f3 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function3");
			odb.Store(f1);
			odb.Store(f2);
			odb.Store(f3);
			NeoDatis.Odb.OID id = odb.GetObjectId(f3);
			odb.Close();
			try
			{
				odb = Open(baseName);
				NeoDatis.Odb.Test.VO.Login.Function f3bis = (NeoDatis.Odb.Test.VO.Login.Function)
					odb.GetObjectFromId(id);
				odb.Delete(f3bis);
				odb.Close();
				odb = Open(baseName);
				NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
					));
				odb.Close();
				AssertEquals(2, l.Count);
			}
			catch (NeoDatis.Odb.ODBRuntimeException)
			{
				odb.Close();
				DeleteBase(baseName);
			}
		}

		/// <summary>
		/// Test : delete the last object and insert a new one in the same
		/// transaction - detected by Alessandra
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test8()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.Test.VO.Login.Function f3 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function3");
			odb.Store(f1);
			odb.Store(f2);
			odb.Store(f3);
			NeoDatis.Odb.OID id = odb.GetObjectId(f3);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f3bis = (NeoDatis.Odb.Test.VO.Login.Function)
				odb.GetObjectFromId(id);
			odb.Delete(f3bis);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("last function"));
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			odb.Close();
			AssertEquals(3, l.Count);
		}

		/// <summary>
		/// Test : delete the last object and insert a new one in another transaction
		/// - detected by Alessandra
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test9()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.Test.VO.Login.Function f3 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function3");
			odb.Store(f1);
			odb.Store(f2);
			odb.Store(f3);
			NeoDatis.Odb.OID id = odb.GetObjectId(f3);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f3bis = (NeoDatis.Odb.Test.VO.Login.Function)
				odb.GetObjectFromId(id);
			odb.Delete(f3bis);
			odb.Close();
			odb = Open(baseName);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("last function"));
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			odb.Close();
			AssertEquals(3, l.Count);
		}

		/// <summary>Test : delete the unique object</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test10()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			long size = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count;
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			odb.Store(f1);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1bis = (NeoDatis.Odb.Test.VO.Login.Function)
				odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).GetFirst();
			odb.Delete(f1bis);
			odb.Close();
			odb = Open(baseName);
			AssertEquals(size, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count
				);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("last function"));
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			odb.Close();
			AssertEquals(size + 1, l.Count);
		}

		/// <summary>Test : delete the unique object</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test11()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			long size = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Login.Function)));
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			odb.Store(f1);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1bis = (NeoDatis.Odb.Test.VO.Login.Function)
				odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).GetFirst();
			odb.Delete(f1bis);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("last function"));
			odb.Close();
			odb = Open(baseName);
			AssertEquals(size + 1, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)
				).Count);
			odb.Close();
		}

		/// <summary>
		/// Bug detected by Olivier using the ODBMainExplorer, deleting many objects
		/// without commiting,and commiting at the end
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test12()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.Test.VO.Login.Function f3 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function3");
			odb.Store(f1);
			odb.Store(f2);
			odb.Store(f3);
			NeoDatis.Odb.OID idf1 = odb.GetObjectId(f1);
			NeoDatis.Odb.OID idf2 = odb.GetObjectId(f2);
			NeoDatis.Odb.OID idf3 = odb.GetObjectId(f3);
			odb.Close();
			try
			{
				odb = Open(baseName);
				odb.DeleteObjectWithId(idf3);
				odb.DeleteObjectWithId(idf2);
				odb.Close();
				odb = Open(baseName);
				NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
					));
				odb.Close();
				AssertEquals(1, l.Count);
			}
			catch (NeoDatis.Odb.ODBRuntimeException e)
			{
				DeleteBase(baseName);
				throw;
			}
		}

		/// <summary>
		/// Bug detected by Olivier using the ODBMainExplorer, deleting many objects
		/// without commiting,and commiting at the end
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test13()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			DeleteBase(baseName);
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.Test.VO.Login.Function f3 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function3");
			odb.Store(f1);
			odb.Store(f2);
			odb.Store(f3);
			NeoDatis.Odb.OID idf1 = odb.GetObjectId(f1);
			NeoDatis.Odb.OID idf2 = odb.GetObjectId(f2);
			NeoDatis.Odb.OID idf3 = odb.GetObjectId(f3);
			long p1 = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine(odb).GetObjectReader
				().GetObjectPositionFromItsOid(idf1, true, false);
			long p2 = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine(odb).GetObjectReader
				().GetObjectPositionFromItsOid(idf2, true, false);
			long p3 = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine(odb).GetObjectReader
				().GetObjectPositionFromItsOid(idf3, true, false);
			odb.Close();
			try
			{
				odb = Open(baseName);
				f1 = (NeoDatis.Odb.Test.VO.Login.Function)odb.GetObjectFromId(idf1);
				f2 = (NeoDatis.Odb.Test.VO.Login.Function)odb.GetObjectFromId(idf2);
				f3 = (NeoDatis.Odb.Test.VO.Login.Function)odb.GetObjectFromId(idf3);
				odb.Delete(f3);
				odb.Delete(f2);
				odb.Close();
				odb = Open(baseName);
				NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
					));
				odb.Close();
				AssertEquals(1, l.Count);
			}
			catch (NeoDatis.Odb.ODBRuntimeException e)
			{
				DeleteBase(baseName);
				throw;
			}
			DeleteBase(baseName);
		}

		/// <summary>creates 5 objects,commit.</summary>
		/// <remarks>
		/// creates 5 objects,commit. Then create 2 new objects and delete 4 existing
		/// objects without committing,and committing at the end
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test14()
		{
			if (!isLocal)
			{
				return;
			}
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.Test.VO.Login.Function f3 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function3");
			NeoDatis.Odb.Test.VO.Login.Function f4 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function4");
			NeoDatis.Odb.Test.VO.Login.Function f5 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function5");
			odb.Store(f1);
			odb.Store(f2);
			odb.Store(f3);
			odb.Store(f4);
			odb.Store(f5);
			AssertEquals(5, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			try
			{
				odb = Open(baseName);
				NeoDatis.Odb.Test.VO.Login.Function f6 = new NeoDatis.Odb.Test.VO.Login.Function(
					"function6");
				NeoDatis.Odb.Test.VO.Login.Function f7 = new NeoDatis.Odb.Test.VO.Login.Function(
					"function7");
				odb.Store(f6);
				odb.Store(f7);
				AssertEquals(7, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
				NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
					));
				int i = 0;
				while (objects.HasNext() && i < 4)
				{
					odb.Delete(objects.Next());
					i++;
				}
				AssertEquals(3, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
				odb.Close();
				odb = Open(baseName);
				AssertEquals(3, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
				objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function));
				// println(objects);
				AssertEquals("function5", ((NeoDatis.Odb.Test.VO.Login.Function)objects.Next()).GetName
					());
				AssertEquals("function6", ((NeoDatis.Odb.Test.VO.Login.Function)objects.Next()).GetName
					());
				AssertEquals("function7", ((NeoDatis.Odb.Test.VO.Login.Function)objects.Next()).GetName
					());
				odb.Close();
			}
			catch (NeoDatis.Odb.ODBRuntimeException e)
			{
				DeleteBase(baseName);
				throw;
			}
			DeleteBase(baseName);
		}

		/// <summary>creates 2 objects.</summary>
		/// <remarks>creates 2 objects. Delete them. And create 2 new objects</remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test15()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			odb.Store(f1);
			odb.Store(f2);
			AssertEquals(2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Delete(f1);
			odb.Delete(f2);
			AssertEquals(0, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Store(f1);
			odb.Store(f2);
			AssertEquals(2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			odb = Open(baseName);
			AssertEquals(2, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count
				);
			odb.Close();
			DeleteBase(baseName);
		}

		/// <summary>creates 2 objects.</summary>
		/// <remarks>creates 2 objects. Delete them by oid. And create 2 new objects</remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test15_by_oid()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.OID oid1 = odb.Store(f1);
			NeoDatis.Odb.OID oid2 = odb.Store(f2);
			AssertEquals(2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.DeleteObjectWithId(oid1);
			odb.DeleteObjectWithId(oid2);
			AssertEquals(0, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Store(f1);
			odb.Store(f2);
			AssertEquals(2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			odb = Open(baseName);
			AssertEquals(2, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count
				);
			odb.Close();
			DeleteBase(baseName);
		}

		/// <summary>creates 2 objects.</summary>
		/// <remarks>creates 2 objects. Delete them by oid. And create 2 new objects</remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test15_by_oid_2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			DeleteBase(baseName);
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.OID oid1 = odb.Store(f1);
			NeoDatis.Odb.OID oid2 = odb.Store(f2);
			AssertEquals(2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			odb = Open(baseName);
			odb.DeleteObjectWithId(oid1);
			odb.DeleteObjectWithId(oid2);
			AssertEquals(0, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Store(f1);
			odb.Store(f2);
			AssertEquals(2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			odb = Open(baseName);
			AssertEquals(2, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count
				);
			odb.Close();
			DeleteBase(baseName);
		}

		/// <summary>creates x objects.</summary>
		/// <remarks>creates x objects. Delete them. And create x new objects</remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test16()
		{
			string baseName = GetBaseName();
			int size = isLocal ? 10000 : 100;
			NeoDatis.Odb.ODB odb = null;
			DeleteBase(baseName);
			odb = Open(baseName);
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function" + i));
			}
			AssertEquals(size, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			for (int i = 0; i < size; i++)
			{
				odb.DeleteObjectWithId(oids[i]);
			}
			AssertEquals(0, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function" + i));
			}
			AssertEquals(size, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			odb = Open(baseName);
			AssertEquals(size, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count
				);
			odb.Close();
			DeleteBase(baseName);
		}

		/// <summary>creates 3 objects.</summary>
		/// <remarks>creates 3 objects. Delete the 2th. And create 3 new objects</remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test17()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			DeleteBase(baseName);
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.Test.VO.Login.Function f3 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			odb.Store(f1);
			odb.Store(f2);
			odb.Store(f3);
			AssertEquals(3, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Delete(f2);
			AssertEquals(2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			// odb.store(f1);
			odb.Store(f2);
			// odb.store(f3);
			AssertEquals(3, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			odb = Open(baseName);
			AssertEquals(3, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count
				);
			odb.Close();
			DeleteBase(baseName);
		}

		/// <summary>creates 3 objects.</summary>
		/// <remarks>
		/// creates 3 objects. commit. Creates 3 new . Delete the 2th commited. And
		/// create 3 new objects
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test18()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			DeleteBase(baseName);
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.Test.VO.Login.Function f3 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			NeoDatis.Odb.OID oid1 = odb.Store(f1);
			NeoDatis.Odb.OID oid2 = odb.Store(f2);
			NeoDatis.Odb.OID oid3 = odb.Store(f3);
			AssertEquals(3, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			odb = Open(baseName);
			odb.DeleteObjectWithId(oid2);
			AssertEquals(2, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			// odb.store(f1);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f11"));
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f12"));
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f13"));
			// odb.store(f3);
			AssertEquals(5, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function))));
			odb.Close();
			odb = Open(baseName);
			AssertEquals(5, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count
				);
			odb.Close();
			DeleteBase(baseName);
		}

		/// <summary>Stores an object, closes the base.</summary>
		/// <remarks>
		/// Stores an object, closes the base. Loads the object, gets its oid and
		/// delete by oid.
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test19()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			odb.Store(f1);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				));
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)objects
				.GetFirst();
			NeoDatis.Odb.OID oid = odb.GetObjectId(f2);
			odb.DeleteObjectWithId(oid);
			AssertEquals(0, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count
				);
			odb.Close();
			odb = Open(baseName);
			objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function));
			AssertEquals(0, objects.Count);
		}

		/// <summary>
		/// Stores on object and close database then Stores another object, commits
		/// without closing.
		/// </summary>
		/// <remarks>
		/// Stores on object and close database then Stores another object, commits
		/// without closing. Loads the object, gets its oid and delete by oid. In the
		/// case the commit has no write actions. And there was a bug : when there is
		/// no write actions, the commit process is much more simple! but in this the
		/// cache was not calling the transaction.clear and this was a reason for
		/// some connected/unconnected zone problem! (step14 of the turotial.)
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test20()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f0 = new NeoDatis.Odb.Test.VO.Login.Function(
				"1function0");
			odb.Store(f0);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			odb.Store(f1);
			odb.Commit();
			NeoDatis.Odb.Objects objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Like("name", "func%")));
			AssertEquals(1, objects.Count);
			NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)objects
				.GetFirst();
			NeoDatis.Odb.OID oid = odb.GetObjectId(f2);
			odb.DeleteObjectWithId(oid);
			AssertEquals(1, odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Count
				);
			odb.Close();
			odb = Open(baseName);
			objects = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function));
			AssertEquals(1, objects.Count);
		}

		/// <summary>
		/// Bug when deleting the first object of unconnected zone when commited zone
		/// already have at least one object.
		/// </summary>
		/// <remarks>
		/// Bug when deleting the first object of unconnected zone when commited zone
		/// already have at least one object.
		/// Detected running the polePosiiton Bahrain circuit.
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test21()
		{
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f0 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function0");
			odb.Store(f0);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Test.VO.Login.Function f1 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function1");
			odb.Store(f1);
			NeoDatis.Odb.Test.VO.Login.Function f2 = new NeoDatis.Odb.Test.VO.Login.Function(
				"function2");
			odb.Store(f2);
			odb.Delete(f1);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function)));
			AssertEquals(2, objects.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test22Last_toCheckDuration()
		{
			long duration = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs() - start;
			long d = 2200;
			if (!isLocal)
			{
				d = 2700;
			}
			Println("duration=" + duration);
			if (testPerformance && duration > d)
			{
				Fail("Duration is higher than " + d + " : " + duration);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			// deleteBase("t-delete12.neodatis");
			DeleteBase("t-delete1.neodatis");
		}
	}
}
