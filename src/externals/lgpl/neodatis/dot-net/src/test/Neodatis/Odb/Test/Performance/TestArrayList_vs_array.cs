using NeoDatis.Tool.Wrappers;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Performance
{
	/// <author>olivier</author>
	[TestFixture]
    public class TestArrayList_vs_array : NeoDatis.Odb.Test.ODBTest
	{
		[Test]
        public virtual void Test1()
		{
			int size = 100000;
			int[] arrayOfInts = new int[size];
			System.Collections.Generic.IList<int> listOfInts = new System.Collections.Generic.List
				<int>(size);
			long startArray = OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				arrayOfInts[i] = i;
			}
			for (int i = 0; i < size; i++)
			{
				int ii = arrayOfInts[i];
			}
            long endArray = OdbTime.GetCurrentTimeInMs();
            long startList = OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				listOfInts.Add(i);
			}
			for (int i = 0; i < size; i++)
			{
				int ii = listOfInts[i];
			}
            long endList = OdbTime.GetCurrentTimeInMs();
			Println("Time for array = " + (endArray - startArray));
			Println("Time for list = " + (endList - startList));
		}
	}
}
