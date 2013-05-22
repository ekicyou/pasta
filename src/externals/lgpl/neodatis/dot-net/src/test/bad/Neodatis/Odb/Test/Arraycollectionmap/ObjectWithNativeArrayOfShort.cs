namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	public class ObjectWithNativeArrayOfShort
	{
		private string name;

		private short[] numbers;

		public ObjectWithNativeArrayOfShort(string name, short[] numbers) : base()
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

		public virtual short[] GetNumbers()
		{
			return numbers;
		}

		public virtual void SetNumbers(short[] numbers)
		{
			this.numbers = numbers;
		}
	}
}
