namespace NeoDatis.Odb.Test.Performance
{
	/// <author>olivier</author>
	public class TestArrayList_vs_array : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			int size = 100000;
			int[] arrayOfInts = new int[size];
			System.Collections.Generic.IList<int> listOfInts = new System.Collections.Generic.List
				<int>(size);
			long startArray = Sharpen.Runtime.CurrentTimeMillis();
			for (int i = 0; i < size; i++)
			{
				arrayOfInts[i] = i;
			}
			for (int i = 0; i < size; i++)
			{
				int ii = arrayOfInts[i];
			}
			long endArray = Sharpen.Runtime.CurrentTimeMillis();
			long startList = Sharpen.Runtime.CurrentTimeMillis();
			for (int i = 0; i < size; i++)
			{
				listOfInts.Add(i);
			}
			for (int i = 0; i < size; i++)
			{
				int ii = listOfInts[i];
			}
			long endList = Sharpen.Runtime.CurrentTimeMillis();
			Println("Time for array = " + (endArray - startArray));
			Println("Time for list = " + (endList - startList));
		}
	}
}
