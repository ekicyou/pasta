using NUnit.Framework;
namespace NeoDatis.Odb.Test.Lookup
{
	/// <author>olivier</author>
	[TestFixture]
    public class TestLookup : NeoDatis.Odb.Test.ODBTest
	{
		[Test]
        public virtual void Test1()
		{
			NeoDatis.Odb.Core.Lookup.ILookup lookup = new NeoDatis.Odb.Core.Lookup.LookupImpl
				();
			lookup.Set("oid1", "Ol√° chico");
			string s = (string)lookup.Get("oid1");
			AssertEquals("Ol√° chico", s);
		}

		[Test]
        public virtual void Test2()
		{
			NeoDatis.Odb.Core.Lookup.ILookup lookup = NeoDatis.Odb.Core.Lookup.LookupFactory.
				Get("test");
			lookup.Set("oid1", "Ol√° chico");
			lookup = NeoDatis.Odb.Core.Lookup.LookupFactory.Get("test");
			string s = (string)lookup.Get("oid1");
			AssertEquals("Ol√° chico", s);
		}
	}
}
