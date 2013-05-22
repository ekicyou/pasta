using NUnit.Framework;
namespace NeoDatis.Odb.Test.Trigger
{
	public class ObjectWithAutoIncrementId
	{
		private string name;

		private long id;

		public ObjectWithAutoIncrementId(string name)
		{
			this.name = name;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual long GetId()
		{
			return id;
		}

		public virtual void SetId(long id)
		{
			this.id = id;
		}
	}
}
