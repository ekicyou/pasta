using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Core.Query.Criteria;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Tool.Wrappers;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Acid
{
	[TestFixture]
    public class TestStopEngineWithoutCommit : NeoDatis.Odb.Test.ODBTest
	{
		private bool simpleObject;

		private NeoDatis.Odb.Test.ODBTest test = new NeoDatis.Odb.Test.ODBTest();

		[Test]
        public virtual void Test1()
		{
		}

		// just to avoid junit warning
		/// <exception cref="System.Exception"></exception>
		public virtual void T1estA1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			odb.Store(GetInstance("f1"));
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estA2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			AssertEquals(0, odb.GetObjects<Function>().Count
				);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estB1()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			odb.Store(GetInstance("f1"));
			odb.Commit();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estB2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int size = 0;
			if (simpleObject)
			{
				size = odb.GetObjects<Function>().Count;
			}
			else
			{
				size = odb.GetObjects<User>().Count;
			}
			if (size != 1)
			{
				throw new System.Exception("Size should be " + 1 + " and it is " + size);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estC1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int size = 1;
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(GetInstance("f" + i));
			}
			for (int i = 0; i < size; i++)
			{
				odb.DeleteObjectWithId(oids[i]);
			}
		}

		private object GetInstance(string @string)
		{
			if (simpleObject)
			{
				return new NeoDatis.Odb.Test.VO.Login.Function(@string);
			}
			NeoDatis.Odb.Test.VO.Login.Profile p = new NeoDatis.Odb.Test.VO.Login.Profile(@string
				);
			p.AddFunction(new NeoDatis.Odb.Test.VO.Login.Function("function " + @string + "1"
				));
			p.AddFunction(new NeoDatis.Odb.Test.VO.Login.Function("function " + @string + "2"
				));
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User(@string
				, "email" + @string, p);
			return user;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estC2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			AssertEquals(0, odb.GetObjects<Function>().Count
				);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estD1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int size = 1000;
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(GetInstance("f" + i));
			}
			for (int i = 0; i < size; i++)
			{
				odb.DeleteObjectWithId(oids[i]);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estD2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			AssertEquals(0, odb.GetObjects<Function>().Count
				);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estE1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int size = 1000;
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(GetInstance("f" + i));
				if (simpleObject)
				{
					NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
						GetObjectFromId(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
				}
				else
				{
					NeoDatis.Odb.Test.VO.Login.User f = (NeoDatis.Odb.Test.VO.Login.User)odb.GetObjectFromId
						(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
				}
				odb.DeleteObjectWithId(oids[i]);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estE2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			AssertEquals(0, odb.GetObjects<Function>().Count
				);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estF1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int size = 1000;
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(GetInstance("f" + i));
				if (simpleObject)
				{
					NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
						GetObjectFromId(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
				}
				else
				{
					NeoDatis.Odb.Test.VO.Login.User f = (NeoDatis.Odb.Test.VO.Login.User)odb.GetObjectFromId
						(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
				}
			}
			for (int i = 0; i < size; i++)
			{
				object o = odb.GetObjectFromId(oids[i]);
				odb.Delete(o);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estF2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				AssertEquals(0, odb.GetObjects<Function>().Count
					);
			}
			else
			{
				AssertEquals(0, odb.GetObjects<User>().Count);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estG1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int size = 1000;
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(GetInstance("f" + i));
				if (simpleObject)
				{
					NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
						GetObjectFromId(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
				}
				else
				{
					NeoDatis.Odb.Test.VO.Login.User f = (NeoDatis.Odb.Test.VO.Login.User)odb.GetObjectFromId
						(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
				}
			}
			odb.Commit();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estG2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int size = 1000;
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(GetInstance("f" + i));
				if (simpleObject)
				{
					NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
						GetObjectFromId(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
				}
				else
				{
					NeoDatis.Odb.Test.VO.Login.User f = (NeoDatis.Odb.Test.VO.Login.User)odb.GetObjectFromId
						(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
					odb.Store(f);
				}
			}
			for (int i = 0; i < size; i++)
			{
				object o = null;
				o = odb.GetObjectFromId(oids[i]);
				odb.Delete(o);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estG3()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				AssertEquals(1000, odb.GetObjects<Function>().Count
					);
			}
			else
			{
				AssertEquals(1000, odb.GetObjects<User>().Count);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estH1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int size = 1000;
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(GetInstance("f" + i));
				if (simpleObject)
				{
					NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
						GetObjectFromId(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
					odb.Delete(f);
					oids[i] = odb.Store(f);
					odb.Delete(f);
					oids[i] = odb.Store(f);
					odb.Delete(f);
					oids[i] = odb.Store(f);
				}
				else
				{
					NeoDatis.Odb.Test.VO.Login.User f = (NeoDatis.Odb.Test.VO.Login.User)odb.GetObjectFromId
						(oids[i]);
					f.SetName("function " + i);
					odb.Store(f);
					odb.Delete(f);
					oids[i] = odb.Store(f);
					odb.Delete(f);
					oids[i] = odb.Store(f);
					odb.Delete(f);
					oids[i] = odb.Store(f);
				}
			}
			for (int i = 0; i < size; i++)
			{
				object o = odb.GetObjectFromId(oids[i]);
				odb.Delete(o);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estH2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				AssertEquals(0, odb.GetObjects<Function>().Count
					);
			}
			else
			{
				AssertEquals(0, odb.GetObjects<User>().Count);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estI1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			odb.Store(GetInstance("f1"));
			odb.Store(GetInstance("f2"));
			odb.Store(GetInstance("f3"));
			odb.Close();
			odb = test.Open("acid1");
			object o = GetInstance("f4");
			odb.Store(o);
			odb.Delete(o);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estI2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				AssertEquals(3, odb.GetObjects<Function>().Count
					);
			}
			else
			{
				AssertEquals(3, odb.GetObjects<User>().Count);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estJ1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			odb.Store(GetInstance("f1"));
			odb.Store(GetInstance("f2"));
			odb.Store(GetInstance("f3"));
			odb.Commit();
			object o = GetInstance("f4");
			odb.Store(o);
			odb.Delete(o);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estJ2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				AssertEquals(3, odb.GetObjects<Function>().Count
					);
			}
			else
			{
				AssertEquals(3, odb.GetObjects<User>().Count);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estK1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			odb.Store(GetInstance("f1"));
			odb.Store(GetInstance("f2"));
			NeoDatis.Odb.OID oid = odb.Store(GetInstance("f3"));
			odb.Commit();
			object o = odb.GetObjectFromId(oid);
			odb.Delete(o);
			odb.Rollback();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estK2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				AssertEquals(3, odb.GetObjects<Function>().Count
					);
			}
			else
			{
				AssertEquals(3, odb.GetObjects<User>().Count);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estL1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			odb.Store(GetInstance("f1"));
			odb.Store(GetInstance("f2"));
			NeoDatis.Odb.OID oid = odb.Store(GetInstance("f3"));
			odb.Commit();
			object o = odb.GetObjectFromId(oid);
			if (simpleObject)
			{
				NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)o;
				f.SetName("flksjdfjs;dfsljflsjflksjfksjfklsdjfksjfkalsjfklsdjflskd");
				odb.Store(f);
			}
			else
			{
				NeoDatis.Odb.Test.VO.Login.User f = (NeoDatis.Odb.Test.VO.Login.User)o;
				f.SetName("flksjdfjs;dfsljflsjflksjfksjfklsdjfksjfkalsjfklsdjflskd");
				odb.Store(f);
			}
			odb.Rollback();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estL2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				AssertEquals(3, odb.GetObjects<Function>().Count
					);
			}
			else
			{
				AssertEquals(3, odb.GetObjects<User>().Count);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estM1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int size = 1;
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(GetInstance("f" + i));
			}
			for (int i = 0; i < size; i++)
			{
				odb.DeleteObjectWithId(oids[i]);
			}
			odb.Rollback();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estM2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				AssertEquals(0, odb.GetObjects<Function>().Count
					);
			}
			else
			{
				AssertEquals(0, odb.GetObjects<User>().Count);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estN1()
		{
			test.DeleteBase("acid1");
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			for (int i = 0; i < 10; i++)
			{
				odb.Store(GetInstance("f" + i));
			}
			odb.Close();
			odb = test.Open("acid1");
			odb.Store(GetInstance("f1000"));
			odb.Commit();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estN2()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>(new CriteriaQuery(Where.Equal("name", "f1000")));
				Function f = objects.GetFirst();
				f.SetName("new name");
				odb.Store(f);
			}
			else
			{
				NeoDatis.Odb.Objects<User> objects = odb.GetObjects<User>(new CriteriaQuery(Where.Equal("name", "f1000")));
				User f = objects.GetFirst();
				f.SetName("new name");
				odb.Store(f);
			}
			odb.Commit();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estN3()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			if (simpleObject)
			{
				NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>(new CriteriaQuery(Where.Equal("name", "new name")));
				odb.Delete(objects.GetFirst());
			}
			else
			{
				NeoDatis.Odb.Objects<User> objects = odb.GetObjects<User>(new CriteriaQuery(Where.Equal("name", "new name")));
				odb.Delete(objects.GetFirst());
			}
			odb.Commit();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estN4()
		{
			NeoDatis.Odb.ODB odb = test.Open("acid1");
			int nb = 0;
			if (simpleObject)
			{
				NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>(new CriteriaQuery(Where.Equal("name", "f1000")));
				nb = objects.Count;
			}
			else
			{
				NeoDatis.Odb.Objects<User> objects = odb.GetObjects<User>(new CriteriaQuery(Where.Equal("name", "f1000")));
				nb = objects.Count;
			}
			if (nb != 0)
			{
				throw new System.Exception("Object f1000 still exist :-(");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Execute(string[] args)
		{
			string step = args[0];
			simpleObject = args[1].Equals("simple");
            System.Reflection.MethodInfo method = null;// OdbReflection.GetMethods(this.GetType(), step, new System.Type[0]);
			try
			{
				method.Invoke(this, new object[0]);
				TestOk(step);
			}
			catch (System.Exception e)
			{
				// println("Error while calling " + step);
				TestBad(step, e);
			}
		}

		// e.printStackTrace();
		private void TestBad(string step, System.Exception e)
		{
			Println(step + " Not ok " + e.InnerException.Message);
			
		}

		private void TestOk(string step)
		{
			Println(step + " Ok ");
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Acid.TestStopEngineWithoutCommit tf = new NeoDatis.Odb.Test.Acid.TestStopEngineWithoutCommit
				();
			try
			{
				tf.Execute(args);
			}
			catch (System.Exception e)
			{
			}
		}
	}
}
