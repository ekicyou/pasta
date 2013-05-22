using NUnit.Framework;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	public class ObjectWithNativeArrayOfInt
	{
		private string name;

		private int[] numbers;

		public ObjectWithNativeArrayOfInt(string name, int[] numbers) : base()
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

		public virtual int[] GetNumbers()
		{
			return numbers;
		}

		public virtual void SetNumbers(int[] numbers)
		{
			this.numbers = numbers;
		}

		public virtual void SetNumber(int index, int value)
		{
			this.numbers[index] = value;
		}

		public virtual int GetNumber(int i)
		{
			return this.numbers[i];
		}
	}
}
