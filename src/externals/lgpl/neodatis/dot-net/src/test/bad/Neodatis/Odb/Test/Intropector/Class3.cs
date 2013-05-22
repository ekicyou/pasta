namespace NeoDatis.Odb.Test.Intropector
{
	public class Class3
	{
		public Class3(string name32)
		{
			this.name3 = name32;
		}

		private string name3;

		public virtual string GetName3()
		{
			return name3;
		}

		public virtual void SetName3(string name3)
		{
			this.name3 = name3;
		}
	}
}
