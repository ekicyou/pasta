namespace NeoDatis.Odb.Test.Performance
{
	public class SimpleObject
	{
		public static int nbgc = 0;

		private string name;

		private int duration;

		private System.DateTime date;

		public SimpleObject(string name, int duration, System.DateTime date)
		{
			this.name = name;
			this.duration = duration;
			this.date = date;
		}

		public SimpleObject()
		{
		}

		public virtual System.DateTime GetDate()
		{
			return date;
		}

		public virtual void SetDate(System.DateTime date)
		{
			this.date = date;
		}

		public virtual int GetDuration()
		{
			return duration;
		}

		public virtual void SetDuration(int duration)
		{
			this.duration = duration;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		~SimpleObject()
		{
			nbgc++;
		}
	}
}
