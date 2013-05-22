using NUnit.Framework;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	public class ObjectWithNativeArrayOfBigDecimal
	{
		private string name;

		private System.Decimal[] numbers;

		public ObjectWithNativeArrayOfBigDecimal(string name, System.Decimal[] numbers) : 
			base()
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

		public virtual System.Decimal[] GetNumbers()
		{
			return numbers;
		}

		public virtual System.Decimal GetNumber(int index)
		{
			return numbers[index];
		}

		public virtual void SetNumbers(System.Decimal[] numbers)
		{
			this.numbers = numbers;
		}

		public virtual void SetNumber(int index, System.Decimal bd)
		{
			numbers[index] = bd;
		}
	}
}
