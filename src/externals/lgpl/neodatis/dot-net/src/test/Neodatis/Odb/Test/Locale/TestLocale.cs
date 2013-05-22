using NUnit.Framework;
namespace NeoDatis.Odb.Test.Locale
{
	/// <author>olivier</author>
	[TestFixture]
    public class TestLocale : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="Java.IO.UnsupportedEncodingException"></exception>
		[Test]
        public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			Java.Util.Locale defaultLocale = Java.Util.Locale.GetDefault();
			try
			{
				odb = Open(baseName);
				NeoDatis.Odb.OdbConfiguration.SetLatinDatabaseCharacterEncoding();
				Java.Util.Locale brLocale = new Java.Util.Locale("pt", "BR");
				Java.Util.Locale.SetDefault(brLocale);
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc.SetBigDecimal1(new System.Decimal(5.3));
				Println(tc.GetBigDecimal1().ToString());
				odb.Store(tc);
				odb.Close();
				odb = Open(baseName);
				NeoDatis.Odb.Objects<NeoDatis.Odb.Test.VO.Attribute.TestClass> objects = odb.GetObjects
					(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass));
				AssertEquals(1, objects.Count);
				AssertEquals(new System.Decimal(5.3), objects.GetFirst().GetBigDecimal1());
			}
			finally
			{
				if (odb != null && !odb.IsClosed())
				{
					odb.Close();
				}
				NeoDatis.Odb.OdbConfiguration.SetDatabaseCharacterEncoding(null);
				Java.Util.Locale.SetDefault(defaultLocale);
			}
		}

		/// <exception cref="Java.IO.UnsupportedEncodingException"></exception>
		[Test]
        public virtual void Test2()
		{
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			Java.Util.Locale defaultLocale = Java.Util.Locale.GetDefault();
			try
			{
				odb = Open(baseName);
				NeoDatis.Odb.OdbConfiguration.SetLatinDatabaseCharacterEncoding();
				Java.Util.Locale brLocale = new Java.Util.Locale("pt", "BR");
				Java.Util.Locale.SetDefault(brLocale);
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc.SetBigDecimal1(new System.Decimal("5.3"));
				Println(tc.GetBigDecimal1().ToString());
				odb.Store(tc);
				odb.Close();
				odb = Open(baseName);
				NeoDatis.Odb.Objects<NeoDatis.Odb.Test.VO.Attribute.TestClass> objects = odb.GetObjects
					(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass));
				AssertEquals(1, objects.Count);
				AssertEquals(new System.Decimal("5.3"), objects.GetFirst().GetBigDecimal1());
			}
			finally
			{
				if (odb != null && !odb.IsClosed())
				{
					odb.Close();
				}
				NeoDatis.Odb.OdbConfiguration.SetDatabaseCharacterEncoding(null);
				Java.Util.Locale.SetDefault(defaultLocale);
			}
		}
	}
}
