namespace NeoDatis.Odb.Test.Performance
{
	/// <summary>
	/// Test map strategy
	/// We need to cache loaded objects.
	/// </summary>
	/// <remarks>
	/// Test map strategy
	/// We need to cache loaded objects. But some of this loaded objects will be
	/// modified and we need to keep track of the modified object (without
	/// duplication)
	/// What is the best strategy?
	/// 1- having two maps, one for loaded objects and one for save objects. Knowing
	/// that all saved objects are in the loaded objects
	/// 2- having one map, where the value is not the object but an Object Wrapper
	/// that has a boolean to indicate if it has been update and the object
	/// ??
	/// </remarks>
	/// <author>osmadja</author>
	public class TestMapPerf : NeoDatis.Odb.Test.ODBTest
	{
		public static int size = 50000;

		/// <summary>Loading x objects, x/2 are modified, using strategy 1</summary>
		public virtual void Test1()
		{
			NeoDatis.Tool.StopWatch stopWatch = new NeoDatis.Tool.StopWatch();
			stopWatch.Start();
			System.Collections.IDictionary loadedObjects = new NeoDatis.Tool.Wrappers.Map.OdbHashMap
				();
			System.Collections.IDictionary modifiedObjects = new NeoDatis.Tool.Wrappers.Map.OdbHashMap
				();
			NeoDatis.Odb.Test.VO.Login.Function f = null;
			NeoDatis.Odb.OID oid = null;
			for (int i = 0; i < size; i++)
			{
				f = new NeoDatis.Odb.Test.VO.Login.Function("function " + i);
				oid = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(i);
				loadedObjects.Add(oid, f);
				if (i < size / 2)
				{
					modifiedObjects.Add(oid, f);
				}
				if (i % 10000 == 0)
				{
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory("put i", false);
				}
			}
			int j = 0;
			int nbModified = 0;
			// Now get all modified objects
			System.Collections.IEnumerator iterator = modifiedObjects.Keys.GetEnumerator();
			while (iterator.MoveNext())
			{
				oid = (NeoDatis.Odb.OID)iterator.Current;
				object o = modifiedObjects[oid];
				if (j % 10000 == 0)
				{
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory("get i", false);
				}
				j++;
				nbModified++;
			}
			stopWatch.End();
			Println("time for 2 maps =" + stopWatch.GetDurationInMiliseconds());
			AssertEquals(size / 2, nbModified);
		}

		/// <summary>Loading x objects, x/2 are modified, using strategy 2</summary>
		public virtual void Test2()
		{
			NeoDatis.Tool.StopWatch stopWatch = new NeoDatis.Tool.StopWatch();
			stopWatch.Start();
			System.Collections.IDictionary objects = new NeoDatis.Tool.Wrappers.Map.OdbHashMap
				();
			NeoDatis.Odb.Test.VO.Login.Function f = null;
			NeoDatis.Odb.OID oid = null;
			NeoDatis.Odb.Test.Performance.ObjectWrapper ow = null;
			int i = 0;
			for (i = 0; i < size; i++)
			{
				f = new NeoDatis.Odb.Test.VO.Login.Function("function " + i);
				oid = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(i);
				objects.Add(oid, new NeoDatis.Odb.Test.Performance.ObjectWrapper(f, false));
				if (i < size / 2)
				{
					ow = (NeoDatis.Odb.Test.Performance.ObjectWrapper)objects[oid];
					ow.SetModified(true);
				}
				if (i % 10000 == 0)
				{
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory("put i", false);
				}
			}
			i = 0;
			int nbModified = 0;
			// Now get all modified objects
			System.Collections.IEnumerator iterator = objects.Keys.GetEnumerator();
			while (iterator.MoveNext())
			{
				oid = (NeoDatis.Odb.OID)iterator.Current;
				ow = (NeoDatis.Odb.Test.Performance.ObjectWrapper)objects[oid];
				if (ow.IsModified())
				{
					nbModified++;
				}
				if (i % 10000 == 0)
				{
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory("get i", false);
				}
				i++;
			}
			stopWatch.End();
			Println("time for 1 map =" + stopWatch.GetDurationInMiliseconds());
			AssertEquals(size / 2, nbModified);
		}
	}

	internal class ObjectWrapper
	{
		private bool modified;

		private object @object;

		public ObjectWrapper(object @object, bool modified)
		{
			this.@object = @object;
			this.modified = modified;
		}

		public virtual bool IsModified()
		{
			return modified;
		}

		public virtual void SetModified(bool modified)
		{
			this.modified = modified;
		}

		public virtual object GetObject()
		{
			return @object;
		}

		public virtual void SetObject(object @object)
		{
			this.@object = @object;
		}

		public override bool Equals(object obj)
		{
			return @object.Equals(obj);
		}

		public override int GetHashCode()
		{
			return @object.GetHashCode();
		}
	}
}
