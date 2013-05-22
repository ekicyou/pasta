using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Values
{
	/// <author>olivier</author>
	[TestFixture]
    public class TestValuesQueryWithOid : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.Query.Values.Handler handler = new NeoDatis.Odb.Test.Query.Values.Handler
				();
			for (int i = 0; i < 10; i++)
			{
				handler.AddParameter(new NeoDatis.Odb.Test.Query.Values.Parameter("test " + i, "value "
					 + i));
			}
			NeoDatis.Odb.OID oid = odb.Store(handler);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Values.Handler), oid).Field("parameters").Sublist
				("parameters", "sub1", 1, 5, true).Sublist("parameters", "sub2", 1, 10).Size("parameters"
				, "size"));
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
	}
}
