namespace NeoDatis.Test.Btree.Impl.Singlevalue
{
	public class TestBTree2 : NeoDatis.Odb.Test.ODBTest
	{
		internal long nbSolutions;

		internal long total;

		internal long nbFailures;

		internal long maxExecutions;

		internal string failureFileName;

		internal string successFileName;

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
		}

		/// <exception cref="System.Exception"></exception>
		private void TestDelete(int[] numbers)
		{
			NeoDatis.Btree.IBTree btree = GetBTree(2);
			for (int i = 0; i < numbers.Length; i++)
			{
				btree.Insert(numbers[i], "v " + numbers[i]);
			}
			AssertEquals(numbers.Length, btree.GetSize());
			for (int i = 0; i < numbers.Length; i++)
			{
				AssertEquals("v " + (i + 1), btree.Delete(i + 1, "v " + (i + 1)));
			}
			AssertEquals(0, btree.GetSize());
			AssertEquals(1, btree.GetHeight());
			AssertEquals(0, btree.GetRoot().GetNbKeys());
			AssertEquals(0, btree.GetRoot().GetNbChildren());
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Te1stDeleteAll()
		{
			long time = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			failureFileName = "btree-failures." + time + ".txt";
			successFileName = "btree-successes." + time + ".txt";
			int size = 10000;
			total = Factorial(size);
			int[] array = new int[size];
			System.Collections.IList elements = new System.Collections.ArrayList();
			for (int i = 0; i < size; i++)
			{
				elements.Add(i + 1);
			}
			BuildArray(array, elements, 0);
			Println(nbSolutions + " solutions");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDeleteAllRandom()
		{
			long time = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			failureFileName = "btree-failures." + time + ".txt";
			successFileName = "btree-successes." + time + ".txt";
			int size = 1000;
			maxExecutions = 100;
			int nbPossibilities = 3;
			total = (long)System.Math.Pow(nbPossibilities, size);
			int[] array = new int[size];
			System.Collections.IList elements = new System.Collections.ArrayList();
			for (int i = 0; i < size; i++)
			{
				elements.Add(i + 1);
			}
			// buildArray(array, elements, 0);
			BuildRandomArray(array, elements, 0, nbPossibilities);
			Println(nbSolutions + " solutions");
			AssertEquals(0, nbFailures);
		}

		private long Factorial(int value)
		{
			if (value == 1)
			{
				return value;
			}
			return value * Factorial(value - 1);
		}

		/// <exception cref="System.Exception"></exception>
		internal virtual void BuildArray(int[] array, System.Collections.IList elements, 
			int currentPosition)
		{
			if (elements.Count == 0)
			{
				Post(array);
			}
			for (int i = 0; i < elements.Count; i++)
			{
				System.Collections.IList myElements = new System.Collections.ArrayList(elements);
				int e = (int)myElements.Remove(i);
				array[currentPosition] = e;
				BuildArray(array, myElements, currentPosition + 1);
			}
		}

		/// <exception cref="System.Exception"></exception>
		internal virtual void BuildRandomArray(int[] array, System.Collections.IList elements
			, int currentPosition, int nbPossibilites)
		{
			if (elements.Count == 0)
			{
				Post(array);
			}
			for (int i = 0; i < nbPossibilites && i < elements.Count && nbSolutions < maxExecutions
				; i++)
			{
				System.Collections.IList myElements = new System.Collections.ArrayList(elements);
				int elementToPick = (int)(System.Math.Random() * elements.Count);
				int e = (int)myElements.Remove(elementToPick);
				array[currentPosition] = e;
				BuildRandomArray(array, myElements, currentPosition + 1, nbPossibilites);
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		private void Post(int[] array)
		{
			// println(DisplayUtility.ojbectArrayToString(array));
			try
			{
				TestDelete(array);
			}
			catch (System.Exception)
			{
				// storeSuccess(array);
				try
				{
					StoreFailure(array);
				}
				catch (System.IO.IOException e1)
				{
					throw;
				}
			}
			nbSolutions++;
			if (nbSolutions % 1000 == 0)
			{
				Println(nbSolutions + "/" + total + "  -  " + nbFailures + " failures");
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		private void StoreFailure(int[] array)
		{
			/*Java.IO.FileWriter fw = new Java.IO.FileWriter(failureFileName, true);
			fw.Write(NeoDatis.Tool.DisplayUtility.ObjectArrayToString(array));
			fw.Write("\n");
			fw.Close();
			nbFailures++;
             * */
		}

		/// <exception cref="System.IO.IOException"></exception>
		private void StoreSuccess(int[] array)
		{
            /*
			Java.IO.FileWriter fw = new Java.IO.FileWriter(successFileName, true);
			fw.Write(NeoDatis.Tool.DisplayUtility.ObjectArrayToString(array));
			fw.Write("\n");
			fw.Close();
             * */
		}

		private NeoDatis.Btree.IBTree GetBTree(int degree)
		{
			return new NeoDatis.Btree.Impl.Singlevalue.InMemoryBTreeSingleValuePerKey("default"
				, degree);
		}
	}
}
