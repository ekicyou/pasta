using NeoDatis.Odb.Test.Update;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Userpassword
{
	[TestFixture]
    public class TestUserPassword : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestWithoutUserAndPassword()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			odb.Store(new NeoDatis.Odb.Test.Update.MyObject(10, "t1"));
			odb.Close();
			odb = Open(baseName);
			AssertEquals(1, odb.GetObjects<MyObject>().Count);
			odb.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestWithUserAndPassword()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName, "user", "password");
			odb.Store(new NeoDatis.Odb.Test.Update.MyObject(10, "t1"));
			odb.Close();
			try
			{
				odb = Open(baseName);
				Fail("it should have stop for invalid user/password");
			}
			catch (NeoDatis.Odb.ODBAuthenticationRuntimeException)
			{
			}
			// odb.rollback();
			// odb.close();
			// e.printStackTrace();
			odb = Open(baseName, "user", "password");
			AssertEquals(1, odb.GetObjects<MyObject>().Count);
			odb.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestWithoutUserAndPassword2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			odb.Store(new NeoDatis.Odb.Test.Update.MyObject(10, "t1"));
			odb.Close();
			try
			{
				odb = Open(baseName, "user", "password");
				Fail("it should have stop for invalid user/password");
			}
			catch (NeoDatis.Odb.ODBAuthenticationRuntimeException)
			{
			}
			// odb.rollback();
			// odb.close();
			// e.printStackTrace();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestWithoutUserAndPasswordWithAccents()
		{
			// Configuration.setDatabaseCharacterEncoding("UTF-8");
			string baseName = GetBaseName();
			DeleteBase(baseName);
			string user = "user";
			string password = "~^√°√≥√≠√£";
			NeoDatis.Odb.ODB odb = Open(baseName, user, password);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("t1"));
			odb.Close();
			try
			{
				odb = Open(baseName, user, password);
			}
			catch (NeoDatis.Odb.ODBAuthenticationRuntimeException)
			{
				odb.Rollback();
				Fail("User/Password with accents");
			}
			odb.Close();
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestWithoutUserAndPasswordWithAccentsFromPropertyFile()
		{
			// Configuration.setDatabaseCharacterEncoding("UTF-8");
			string baseName = GetBaseName();
			DeleteBase(baseName);
			
			// The test-accent property file is in the test directory
			//Java.Util.ResourceBundle r = Java.Util.ResourceBundle.GetBundle("test-accent");
			//string user = r.GetString("user");
			///string password = r.GetString("password");
			Println(password);
			NeoDatis.Odb.ODB odb = Open(baseName, user, password);
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("t1"));
			odb.Close();
			try
			{
				odb = Open(baseName, "\u00E7\u00E3o", "ol\u00E1 chico");
			}
			catch (NeoDatis.Odb.ODBAuthenticationRuntimeException)
			{
				odb.Rollback();
				Fail("User/Password with accents");
			}
			odb.Close();
			DeleteBase(baseName);
		}
	}
}
