using NUnit.Framework;
namespace NeoDatis.Odb.Test.Update
{
	public class MyObject
	{
		private int size;

		private string name;

		private System.DateTime date;

		public MyObject(int size, string name)
		{
			this.size = size;
			this.name = name;
		}

		public override string ToString()
		{
			return "size=" + size + " - name=" + name + " - time=" + (date == null ? "null" : 
				string.Empty + date.Millisecond);
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual int GetSize()
		{
			return size;
		}

		public virtual void SetSize(int size)
		{
			this.size = size;
		}

		public virtual System.DateTime GetDate()
		{
			return date;
		}

		public virtual void SetDate(System.DateTime date)
		{
			this.date = date;
		}
	}
}
