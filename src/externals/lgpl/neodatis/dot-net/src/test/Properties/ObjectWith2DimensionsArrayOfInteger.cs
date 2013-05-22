using NUnit.Framework;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	public class ObjectWith2DimensionsArrayOfInteger
	{
		private string name;

		private int[][] numbers;

		public ObjectWith2DimensionsArrayOfInteger(string name, int[][] numbers) : base()
		{
			this.name = name;
			this.numbers = numbers;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual int[][] GetNumbers()
		{
			return numbers;
		}

		public virtual int GetNumber(int i, int j)
		{
			return numbers[i][j];
		}

		public virtual void SetNumbers(int[][] numbers)
		{
			this.numbers = numbers;
		}

		public virtual void SetNumber(int i, int j, int value)
		{
			numbers[i][j] = value;
		}
	}
}
