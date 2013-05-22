using NUnit.Framework;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	public class ObjectWithNativeArrayOfDate
	{
		private string name;

		private System.DateTime[] numbers;

		public ObjectWithNativeArrayOfDate(string name, System.DateTime[] numbers) : base
			()
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

		public virtual System.DateTime[] GetNumbers()
		{
			return numbers;
		}

		public virtual System.DateTime GetNumber(int index)
		{
			return numbers[index];
		}

		public virtual void SetNumbers(System.DateTime[] numbers)
		{
			this.numbers = numbers;
		}

		public virtual void SetNumber(int index, System.DateTime bd)
		{
			numbers[index] = bd;
		}
	}
}
