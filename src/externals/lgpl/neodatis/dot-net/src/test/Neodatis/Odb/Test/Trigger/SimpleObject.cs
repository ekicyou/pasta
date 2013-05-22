using NUnit.Framework;
namespace NeoDatis.Odb.Test.Trigger
{
	/// <author>olivier</author>
	public class SimpleObject
	{
		private int id;

		public SimpleObject(int i)
		{
			this.id = i;
		}

		public virtual int GetId()
		{
			return id;
		}

		public virtual void SetId(int id)
		{
			this.id = id;
		}
	}
}
