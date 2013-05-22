using NUnit.Framework;
namespace NeoDatis.Odb.Test.Performance
{
	[TestFixture]
    public class TestInstanceOf : NeoDatis.Odb.Test.ODBTest
	{
		private const int size = 100000000;

		[Test]
        public virtual void TestIfInstanceOf()
		{
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				(null);
			for (int i = 0; i < size; i++)
			{
				if (nnoi is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)
				{
				}
			}
			// println("time instance of=" + (OdbTime.getCurrentTimeInMs()-start));
			AssertTrue(true);
		}

		[Test]
        public virtual void TestIf()
		{
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				(null);
			for (int i = 0; i < size; i++)
			{
				if (nnoi.IsCollectionObject())
				{
				}
			}
			// println("time if=" + (OdbTime.getCurrentTimeInMs()-start));
			AssertTrue(true);
		}
	}
}
