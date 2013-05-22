using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Values
{
	/// <author>olivier</author>
	[TestFixture]
    public class TestSubList : NeoDatis.Odb.Test.ODBTest
	{
		[Test]
        public virtual void TestSubListJava()
		{
			System.Collections.IList l = new System.Collections.ArrayList();
			l.Add("param1");
			l.Add("param2");
			l.Add("param3");
			l.Add("param4");
			int fromIndex = 1;
			int size = 2;
			int endIndex = fromIndex + size;
			System.Collections.IList l2 = l.SubList(fromIndex, endIndex);
			AssertEquals(2, l2.Count);
		}

		/// <summary>Test when size is bigger than list</summary>
		[Test]
        public virtual void TestSubListJava2()
		{
			System.Collections.IList l = new System.Collections.ArrayList();
			l.Add("param1");
			l.Add("param2");
			l.Add("param3");
			l.Add("param4");
			int fromIndex = 1;
			int size = 20;
			int endIndex = fromIndex + size;
			bool throwException = false;
			if (!throwException)
			{
				if (endIndex > l.Count)
				{
					endIndex = l.Count;
				}
			}
			System.Collections.IList l2 = l.SubList(fromIndex, endIndex);
			AssertEquals(3, l2.Count);
		}

		/// <summary>Test when start index is greater than list size</summary>
		[Test]
        public virtual void TestSubListJava3()
		{
			System.Collections.IList l = new System.Collections.ArrayList();
			int fromIndex = 100;
			int size = 20;
			int endIndex = fromIndex + size;
			bool throwException = false;
			if (!throwException)
			{
				if (fromIndex > l.Count - 1)
				{
					fromIndex = 0;
				}
				if (endIndex > l.Count)
				{
					endIndex = l.Count;
				}
			}
			System.Collections.IList l2 = l.SubList(fromIndex, endIndex);
			AssertEquals(0, l2.Count);
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			DeleteBase("valuesSubList");
			NeoDatis.Odb.ODB odb = Open("valuesSubList");
			NeoDatis.Odb.Test.Query.Values.Handler handler = new NeoDatis.Odb.Test.Query.Values.Handler
				();
			for (int i = 0; i < 10; i++)
			{
				handler.AddParameter(new NeoDatis.Odb.Test.Query.Values.Parameter("test " + i, "value "
					 + i));
			}
			odb.Store(handler);
			odb.Close();
			odb = Open("valuesSubList");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Values.Handler)).Field("parameters").Sublist("parameters"
				, "sub1", 1, 5, true).Sublist("parameters", "sub2", 1, 10).Size("parameters", "size"
				));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			System.Collections.IList fulllist = (System.Collections.IList)ov.GetByAlias("parameters"
				);
			AssertEquals(10, fulllist.Count);
			long size = (long)ov.GetByAlias("size");
			AssertEquals(10, size);
			NeoDatis.Odb.Test.Query.Values.Parameter p = (NeoDatis.Odb.Test.Query.Values.Parameter
				)fulllist[0];
			AssertEquals("value 0", p.GetValue());
			NeoDatis.Odb.Test.Query.Values.Parameter p2 = (NeoDatis.Odb.Test.Query.Values.Parameter
				)fulllist[9];
			AssertEquals("value 9", p2.GetValue());
			System.Collections.IList sublist = (System.Collections.IList)ov.GetByAlias("sub1"
				);
			AssertEquals(5, sublist.Count);
			p = (NeoDatis.Odb.Test.Query.Values.Parameter)sublist[0];
			AssertEquals("value 1", p.GetValue());
			p2 = (NeoDatis.Odb.Test.Query.Values.Parameter)sublist[4];
			AssertEquals("value 5", p2.GetValue());
			System.Collections.IList sublist2 = (System.Collections.IList)ov.GetByAlias("sub2"
				);
			AssertEquals(9, sublist2.Count);
			odb.Close();
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test11()
		{
			NeoDatis.Odb.ODB odb = NeoDatis.Odb.ODBFactory.Open("valuesSubList");
			NeoDatis.Odb.Test.Query.Values.Handler handler = new NeoDatis.Odb.Test.Query.Values.Handler
				();
			for (int i = 0; i < 10; i++)
			{
				handler.AddParameter(new NeoDatis.Odb.Test.Query.Values.Parameter("test " + i, "value "
					 + i));
			}
			odb.Store(handler);
			odb.Close();
			odb = Open("valuesSubList");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Values.Handler)).Field("parameters").Sublist("parameters"
				, "sub1", 1, 5, true).Sublist("parameters", "sub2", 1, 10).Size("parameters", "size"
				));
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			// Retrieve Result values
			System.Collections.IList fulllist = (System.Collections.IList)ov.GetByAlias("parameters"
				);
			long size = (long)ov.GetByAlias("size");
			System.Collections.IList sublist = (System.Collections.IList)ov.GetByAlias("sub1"
				);
			odb.Close();
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			DeleteBase("valuesSubList2");
			NeoDatis.Odb.ODB odb = Open("valuesSubList2");
			NeoDatis.Odb.Test.Query.Values.Handler handler = new NeoDatis.Odb.Test.Query.Values.Handler
				();
			for (int i = 0; i < 500; i++)
			{
				handler.AddParameter(new NeoDatis.Odb.Test.Query.Values.Parameter("test " + i, "value "
					 + i));
			}
			NeoDatis.Odb.OID oid = odb.Store(handler);
			odb.Close();
			odb = Open("valuesSubList2");
			NeoDatis.Odb.Test.Query.Values.Handler h = (NeoDatis.Odb.Test.Query.Values.Handler
				)odb.GetObjectFromId(oid);
			Println("size of list = " + h.GetListOfParameters().Count);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Values.Handler)).Sublist("parameters", "sub", 490
				, 5, true).Size("parameters", "size"));
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("time to load sublist of 5 itens from 40000 : " + (end - start));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			System.Collections.IList sublist = (System.Collections.IList)ov.GetByAlias("sub");
			AssertEquals(5, sublist.Count);
			long size = (long)ov.GetByAlias("size");
			AssertEquals(500, size);
			NeoDatis.Odb.Test.Query.Values.Parameter p = (NeoDatis.Odb.Test.Query.Values.Parameter
				)sublist[0];
			AssertEquals("value 490", p.GetValue());
			NeoDatis.Odb.Test.Query.Values.Parameter p2 = (NeoDatis.Odb.Test.Query.Values.Parameter
				)sublist[4];
			AssertEquals("value 494", p2.GetValue());
			odb.Close();
		}

		/// <summary>Using Object representation instead of real object</summary>
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3()
		{
			int sublistSize = 10000;
			DeleteBase("valuesSubList3");
			NeoDatis.Odb.ODB odb = Open("valuesSubList3");
			NeoDatis.Odb.Test.Query.Values.Handler handler = new NeoDatis.Odb.Test.Query.Values.Handler
				();
			for (int i = 0; i < sublistSize; i++)
			{
				handler.AddParameter(new NeoDatis.Odb.Test.Query.Values.Parameter("test " + i, "value "
					 + i));
			}
			odb.Store(handler);
			odb.Close();
			odb = Open("valuesSubList3");
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Values.Handler)).Sublist("parameters", "sub", 9990
				, 5, true);
			q.SetReturnInstance(false);
			NeoDatis.Odb.Values values = odb.GetValues(q);
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("time to load sublist of 5 itens from 40000 : " + (end - start));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			System.Collections.IList sublist = (System.Collections.IList)ov.GetByAlias("sub");
			AssertEquals(5, sublist.Count);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)sublist[0];
			AssertEquals("value 9990", nnoi.GetValueOf("value"));
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi2 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)sublist[4];
			AssertEquals("value 9994", nnoi2.GetValueOf("value"));
			odb.Close();
		}

		/// <summary>Using Object representation instead of real object</summary>
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test5()
		{
			int sublistSize = 400;
			if (!isLocal && !useSameVmOptimization)
			{
				sublistSize = 40;
			}
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.Query.Values.Handler handler = new NeoDatis.Odb.Test.Query.Values.Handler
				();
			for (int i = 0; i < sublistSize; i++)
			{
				handler.AddParameter(new NeoDatis.Odb.Test.Query.Values.Parameter("test " + i, "value "
					 + i));
			}
			odb.Store(handler);
			odb.Close();
			odb = Open("valuesSubList3");
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Values.Handler)).Sublist("parameters", "sub", 0, 
				2, true);
			NeoDatis.Odb.Values values = odb.GetValues(q);
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("time to load sublist of 5 itens for " + sublistSize + " : " + (end - start
				));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			System.Collections.IList sublist = (System.Collections.IList)ov.GetByAlias("sub");
			AssertEquals(2, sublist.Count);
			NeoDatis.Odb.Test.Query.Values.Parameter parameter = (NeoDatis.Odb.Test.Query.Values.Parameter
				)sublist[1];
			AssertEquals("value 1", parameter.GetValue());
			NeoDatis.Odb.OID oid = odb.GetObjectId(parameter);
			Println(oid);
			odb.Close();
		}

		/// <summary>Check if objects of list are known by ODB</summary>
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test6()
		{
			int sublistSize = 400;
			if (!isLocal && !useSameVmOptimization)
			{
				sublistSize = 40;
			}
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.Query.Values.Handler handler = new NeoDatis.Odb.Test.Query.Values.Handler
				();
			for (int i = 0; i < sublistSize; i++)
			{
				handler.AddParameter(new NeoDatis.Odb.Test.Query.Values.Parameter("test " + i, "value "
					 + i));
			}
			odb.Store(handler);
			odb.Close();
			odb = Open("valuesSubList3");
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Values.Handler));
			NeoDatis.Odb.Objects objects = odb.GetObjects(q);
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Test.Query.Values.Handler h = (NeoDatis.Odb.Test.Query.Values.Handler
				)objects.GetFirst();
			NeoDatis.Odb.Test.Query.Values.Parameter parameter = (NeoDatis.Odb.Test.Query.Values.Parameter
				)h.GetListOfParameters()[0];
			AssertEquals("value 0", parameter.GetValue());
			NeoDatis.Odb.OID oid = odb.GetObjectId(parameter);
			AssertNotNull(oid);
			odb.Close();
		}

		[Test]
        public virtual void Test4()
		{
			DeleteBase("sublist4");
			NeoDatis.Odb.ODB odb = Open("sublist4");
			int i = 0;
			System.Collections.IList functions1 = new System.Collections.ArrayList();
			for (i = 0; i < 30; i++)
			{
				functions1.Add(new NeoDatis.Odb.Test.VO.Login.Function("f1-" + i));
			}
			System.Collections.IList functions2 = new System.Collections.ArrayList();
			for (i = 0; i < 60; i++)
			{
				functions2.Add(new NeoDatis.Odb.Test.VO.Login.Function("f2-" + i));
			}
			System.Collections.IList functions3 = new System.Collections.ArrayList();
			for (i = 0; i < 90; i++)
			{
				functions3.Add(new NeoDatis.Odb.Test.VO.Login.Function("f3-" + i));
			}
			NeoDatis.Odb.Test.VO.Login.User user1 = new NeoDatis.Odb.Test.VO.Login.User("User1"
				, "user1@neodtis.org", new NeoDatis.Odb.Test.VO.Login.Profile("profile1", functions1
				));
			NeoDatis.Odb.Test.VO.Login.User user2 = new NeoDatis.Odb.Test.VO.Login.User("User2"
				, "user1@neodtis.org", new NeoDatis.Odb.Test.VO.Login.Profile("profile2", functions2
				));
			NeoDatis.Odb.Test.VO.Login.User user3 = new NeoDatis.Odb.Test.VO.Login.User("User3"
				, "user1@neodtis.org", new NeoDatis.Odb.Test.VO.Login.Profile("profile3", functions3
				));
			odb.Store(user1);
			odb.Store(user2);
			odb.Store(user3);
			odb.Close();
			odb = Open("sublist4");
			NeoDatis.Odb.Test.VO.Login.User u = (NeoDatis.Odb.Test.VO.Login.User)odb.GetObjects
				(typeof(NeoDatis.Odb.Test.VO.Login.User)).GetFirst();
			System.Console.Out.WriteLine(u);
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Profile)).Field("name").Sublist("functions", 
				1, 2, false).Size("functions", "fsize");
			NeoDatis.Odb.Values v = odb.GetValues(q);
			i = 0;
			while (v.HasNext())
			{
				NeoDatis.Odb.ObjectValues ov = v.NextValues();
				string profileName = (string)ov.GetByAlias("name");
				Println(profileName);
				AssertEquals("profile" + (i + 1), profileName);
				AssertEquals(System.Convert.ToInt64(30 * (i + 1)), ov.GetByAlias("fsize"));
				System.Collections.IList l = (System.Collections.IList)ov.GetByAlias("functions");
				Println(l);
				AssertEquals(2, l.Count);
				i++;
			}
			odb.Close();
		}
	}
}
