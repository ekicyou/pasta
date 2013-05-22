using System;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Other
{
	[TestFixture]
    public class TestJavaClassPersistence : NeoDatis.Odb.Test.ODBTest
	{
		public static readonly string DbName = "class.neodatis";

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			DeleteBase(DbName);
			NeoDatis.Odb.ODB odb = Open(DbName);
			odb.Store(new System.Exception("test"));
			odb.Close();
			odb = Open(DbName);
			NeoDatis.Odb.Objects<Exception> l = odb.GetObjects<Exception>();
			odb.Close();
			DeleteBase(DbName);
		}
	}
}
