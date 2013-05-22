using NUnit.Framework;
namespace NeoDatis.Odb.Test.Index
{
	/// <author>olivier</author>
	[TestFixture]
    public class TestIndexesWithValueQueries : NeoDatis.Odb.Test.ODBTest
	{
		[Test]
        public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			int size = 10000;
			try
			{
				odb = Open(baseName);
				odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.VO.Login.Function)).AddIndexOn
					("index1", new string[] { "name" }, true);
				for (int i = 0; i < size; i++)
				{
					odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function " + i));
				}
				odb.Close();
				odb = Open(baseName);
				// build a value query to retrieve only the name of the function
				NeoDatis.Odb.Core.Query.IValuesQuery vq = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "function " + (size - 1))).Field("name");
				NeoDatis.Odb.Values values = odb.GetValues(vq);
				AssertEquals(1, values.Count);
				Println(vq.GetExecutionPlan().GetDetails());
				AssertEquals(true, vq.GetExecutionPlan().UseIndex());
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}
	}
}
