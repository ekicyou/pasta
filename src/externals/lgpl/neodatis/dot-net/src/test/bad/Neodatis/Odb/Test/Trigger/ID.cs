namespace NeoDatis.Odb.Test.Trigger
{
	public class ID
	{
		private string idName;

		private long id;

		public ID(string name, long id)
		{
			this.idName = name;
			this.id = id;
		}

		public virtual long GetNext()
		{
			id++;
			return id;
		}
	}
}
