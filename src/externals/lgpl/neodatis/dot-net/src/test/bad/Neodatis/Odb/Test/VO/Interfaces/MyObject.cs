namespace NeoDatis.Odb.Test.VO.Interfaces
{
	/// <author>olivier</author>
	[System.Serializable]
	public class MyObject
	{
		public MyObject(string name) : base()
		{
			this.name = name;
		}

		private string name;

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}
	}
}
