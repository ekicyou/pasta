using NeoDatis.Odb.Test.VO.Attribute;
using NeoDatis.Odb.Test.List;
using NeoDatis.Odb.Test.VO.Login;
using System.Collections.Generic;
using NUnit.Framework;
using System;
namespace NeoDatis.Odb.Test.Insert
{
    [TestFixture]
	public class StorageEngineTest : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestNonNativeAttributes()
		{
			TestClass tc = new TestClass
				();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetClassIntrospector().Introspect(tc.GetType(), true).GetMainClassInfo
				();
			AssertEquals(0, classInfo.GetAllNonNativeAttributes().Count);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestSimpleInstance()
		{
			DeleteBase("t-simple-instance.neodatis");
			NeoDatis.Odb.ODB odb = Open("t-simple-instance.neodatis");
			TestClass tc1 = new TestClass
				();
			tc1.SetBigDecimal1(new System.Decimal(1.123456));
			tc1.SetBoolean1(true);
			tc1.SetChar1('d');
			tc1.SetDouble1(154.78998989);
			tc1.SetInt1(78964);
			tc1.SetString1("Ola chico como vc est\u00E1 ???");
			tc1.SetDate1(DateTime.Now);
			tc1.SetBoolean2(false);
			TestClass tc2 = new TestClass
				();
			tc2.SetBigDecimal1(new System.Decimal(1.1234565454));
			tc2.SetBoolean1(false);
			tc2.SetChar1('c');
			tc2.SetDouble1(78454.8779);
			tc2.SetInt1(1254);
			tc2.SetString1("Ola chico como ca va ???");
			tc2.SetDate1(DateTime.Now);
			tc2.SetBoolean2(true);
			odb.Store(tc1);
			odb.Store(tc2);
			odb.Close();
			odb = Open("t-simple-instance.neodatis");
			NeoDatis.Odb.Objects<TestClass> l = odb.GetObjects<TestClass>(true);
			TestClass tc12 = l.GetFirst();
			// println("#### " + l.size() + " : " + l);
			AssertEquals(tc1.GetBigDecimal1(), tc12.GetBigDecimal1());
			AssertEquals(tc1.GetString1(), tc12.GetString1());
			AssertEquals(tc1.GetChar1(), tc12.GetChar1());
			AssertEquals(tc1.GetDouble1(), tc12.GetDouble1());
			AssertEquals(tc1.GetInt1(), tc12.GetInt1());
			AssertEquals(tc1.IsBoolean1(), tc12.IsBoolean1());
			AssertEquals(false, tc12.GetBoolean2());
            Console.WriteLine(" Date is " + tc12.GetDate1());
			if (l.Count < 3)
			{
				AssertEquals(tc1.GetDate1(), tc12.GetDate1());
			}
			l.Next();
			TestClass tc22 = (TestClass
				)l.Next();
			AssertEquals(tc2.GetBigDecimal1(), tc22.GetBigDecimal1());
			AssertEquals(tc2.GetString1(), tc22.GetString1());
			AssertEquals(tc2.GetChar1(), tc22.GetChar1());
			AssertEquals(tc2.GetDouble1(), tc22.GetDouble1());
			AssertEquals(tc2.GetInt1(), tc22.GetInt1());
			AssertEquals(tc2.IsBoolean1(), tc22.IsBoolean1());
			AssertEquals(true, tc2.GetBoolean2());
			if (l.Count < 3)
			{
				AssertEquals(tc2.GetDate1(), tc22.GetDate1());
			}
			odb.Close();
		}

		// deleteBase("t-simple-instance.neodatis");
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestSimpleInstanceRetrievingWithNQ()
		{
			if (!isLocal || !useSameVmOptimization)
			{
				return;
			}
			DeleteBase("t-simple-instance.neodatis");
			NeoDatis.Odb.ODB odb = Open("t-simple-instance.neodatis");
			TestClass tc1 = new TestClass
				();
			tc1.SetBigDecimal1(new System.Decimal(1.123456));
			tc1.SetBoolean1(true);
			tc1.SetChar1('d');
			tc1.SetDouble1(154.78998989);
			tc1.SetInt1(78964);
			tc1.SetString1("Ola chico como vc est\u00E1 ???");
			tc1.SetDate1(new System.DateTime());
			tc1.SetBoolean2(false);
			TestClass tc2 = new TestClass
				();
			tc2.SetBigDecimal1(new System.Decimal(1.1234565454));
			tc2.SetBoolean1(false);
			tc2.SetChar1('c');
			tc2.SetDouble1(78454.8779);
			tc2.SetInt1(1254);
			tc2.SetString1("Ola chico como ca va ???");
			tc2.SetDate1(new System.DateTime());
			tc2.SetBoolean2(true);
			odb.Store(tc1);
			odb.Store(tc2);
			odb.Close();
			odb = Open("t-simple-instance.neodatis");
			NeoDatis.Odb.Core.Query.IQuery q = new _SimpleNativeQuery_146();
			NeoDatis.Odb.Objects<TestClass> l = odb.GetObjects<TestClass>(q);
			TestClass tc12 = l.GetFirst();
			// println("#### " + l.size() + " : " + l);
			AssertEquals(tc1.GetBigDecimal1(), tc12.GetBigDecimal1());
			AssertEquals(tc1.GetString1(), tc12.GetString1());
			AssertEquals(tc1.GetChar1(), tc12.GetChar1());
			AssertEquals(tc1.GetDouble1(), tc12.GetDouble1());
			AssertEquals(tc1.GetInt1(), tc12.GetInt1());
			AssertEquals(tc1.IsBoolean1(), tc12.IsBoolean1());
			AssertEquals(false, tc12.GetBoolean2());
			if (l.Count < 3)
			{
				AssertEquals(tc1.GetDate1(), tc12.GetDate1());
			}
			l.Next();
			TestClass tc22 = (TestClass)l.Next();
			AssertEquals(tc2.GetBigDecimal1(), tc22.GetBigDecimal1());
			AssertEquals(tc2.GetString1(), tc22.GetString1());
			AssertEquals(tc2.GetChar1(), tc22.GetChar1());
			AssertEquals(tc2.GetDouble1(), tc22.GetDouble1());
			AssertEquals(tc2.GetInt1(), tc22.GetInt1());
			AssertEquals(tc2.IsBoolean1(), tc22.IsBoolean1());
			AssertEquals(true, tc2.GetBoolean2());
			if (l.Count < 3)
			{
				AssertEquals(tc2.GetDate1(), tc22.GetDate1());
			}
			odb.Close();
			DeleteBase("t-simple-instance.neodatis");
		}

		private sealed class _SimpleNativeQuery_146 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_146()
			{
			}

			public bool Match(TestClass @object)
			{
				return true;
			}
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestSimpleInstanceRetrievingWithNQWithNullBoolean()
		{
			if (!isLocal || !useSameVmOptimization)
			{
				// native must be serializable to be executed in cs mode
				return;
			}
			DeleteBase("t-simple-instance.neodatis");
			NeoDatis.Odb.ODB odb = Open("t-simple-instance.neodatis");
			TestClass tc1 = new TestClass
				();
			tc1.SetBigDecimal1(new System.Decimal(1.123456));
			tc1.SetBoolean1(true);
			tc1.SetChar1('d');
			tc1.SetDouble1(154.78998989);
			tc1.SetInt1(78964);
			tc1.SetString1("Ola chico como vc est√° ???");
			tc1.SetDate1(new System.DateTime());
			tc1.SetBoolean2(false);
			TestClass tc2 = new TestClass
				();
			tc2.SetBigDecimal1(new System.Decimal(1.1234565454));
			tc2.SetBoolean1(false);
			tc2.SetChar1('c');
			tc2.SetDouble1(78454.8779);
			tc2.SetInt1(1254);
			tc2.SetString1("Ola chico como ca va ???");
			tc2.SetDate1(new System.DateTime());
			tc2.SetBoolean2(true);
			odb.Store(tc1);
			odb.Store(tc2);
			odb.Close();
			odb = Open("t-simple-instance.neodatis");
			NeoDatis.Odb.Core.Query.IQuery q = new _SimpleNativeQuery_217();
			NeoDatis.Odb.Objects<TestClass> l = odb.GetObjects<TestClass>(q);
			TestClass tc12 = (TestClass
				)l.GetFirst();
			// println("#### " + l.size() + " : " + l);
			AssertEquals(tc1.GetBigDecimal1(), tc12.GetBigDecimal1());
			AssertEquals(tc1.GetString1(), tc12.GetString1());
			AssertEquals(tc1.GetChar1(), tc12.GetChar1());
			AssertEquals(tc1.GetDouble1(), tc12.GetDouble1());
			AssertEquals(tc1.GetInt1(), tc12.GetInt1());
			AssertEquals(tc1.IsBoolean1(), tc12.IsBoolean1());
			AssertEquals(false, tc12.GetBoolean2());
			if (l.Count < 3)
			{
				AssertEquals(tc1.GetDate1(), tc12.GetDate1());
			}
			l.Next();
			TestClass tc22 = (TestClass
				)l.Next();
			AssertEquals(tc2.GetBigDecimal1(), tc22.GetBigDecimal1());
			AssertEquals(tc2.GetString1(), tc22.GetString1());
			AssertEquals(tc2.GetChar1(), tc22.GetChar1());
			AssertEquals(tc2.GetDouble1(), tc22.GetDouble1());
			AssertEquals(tc2.GetInt1(), tc22.GetInt1());
			AssertEquals(tc2.IsBoolean1(), tc22.IsBoolean1());
			AssertEquals(true, tc2.GetBoolean2());
			if (l.Count < 3)
			{
				AssertEquals(tc2.GetDate1(), tc22.GetDate1());
			}
			odb.Close();
			DeleteBase("t-simple-instance.neodatis");
		}

		private sealed class _SimpleNativeQuery_217 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_217()
			{
			}

			public bool Match(TestClass @object)
			{
				return true;
			}
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestComplexInstance()
		{
			DeleteBase("t-complex-instance.neodatis");
			NeoDatis.Odb.ODB odb = Open("t-complex-instance.neodatis");
			Function login = new Function
				("login");
			Function logout = new Function
				("logout");
			List<Function> functions = new List<Function>();
			functions.Add(login);
			functions.Add(logout);
			Profile profile = new Profile("profile1", functions);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("oliver"
				, "olivier@neodatis.com", profile);
            NeoDatis.Odb.Test.VO.Login.User user22 = new NeoDatis.Odb.Test.VO.Login.User("oliver2"
				, "olivier2@neodatis.com", profile);
			odb.Store(user);
			odb.Store(user22);
			odb.Close();
			odb = Open("t-complex-instance.neodatis");
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.VO.Login.User> l = odb.GetObjects<NeoDatis.Odb.Test.VO.Login.User>(true);
            NeoDatis.Odb.Test.VO.Login.User user2 = l.GetFirst();
			// println("#### " + l.size() + " : " + l);
			AssertEquals(user.GetName(), user2.GetName());
			AssertEquals(user.GetEmail(), user2.GetEmail());
			AssertEquals(user.GetProfile().GetName(), user2.GetProfile().GetName());
			AssertEquals(user.GetProfile().GetFunctions()[0].ToString(), user2.GetProfile().GetFunctions
				()[0].ToString());
			odb.Close();
			DeleteBase("t-complex-instance.neodatis");
		}
	}
}
